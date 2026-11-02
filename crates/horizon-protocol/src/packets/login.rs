//! Login state: offline/online hello, encryption request, compression,
//! login plugin queries and the finished game profile.
use crate::{
    buf::{Bool, Bytes, Ident, Opt, Rest, Str, StrN, Uuid, VarInt},
    packet, packet_enum,
    packets::Empty,
};

packet!(
    /// com.mojang.authlib.properties.Property.
    Property {
        name: String => StrN<64>,
        value: String => Str,
        signature: Option<String> => Opt<StrN<1024>>,
    }
);
packet!(
    /// ByteBufCodecs.GAME_PROFILE.
    GameProfile {
        uuid: u128 => Uuid,
        name: String => StrN<16>,
        properties: Vec<Property> => crate::buf::List<Property>,
    }
);
packet!(Disconnect { reason_json: String => StrN<262144> });
#[derive(Clone, Debug, PartialEq)]
pub struct Hello {
    pub server_id: String,
    pub public_key: Vec<u8>,
    pub challenge: Vec<u8>,
    pub should_authenticate: bool,
}
impl crate::buf::Codec<Hello> for Hello {
    fn read(r: &mut crate::buf::Reader) -> crate::Result<Self> {
        let server_id = <StrN<20> as crate::buf::Codec<String>>::read(r)?;
        let public_key = <Bytes as crate::buf::Codec<Vec<u8>>>::read(r)?;
        let challenge = <Bytes as crate::buf::Codec<Vec<u8>>>::read(r)?;
        // `should_authenticate` was added in 1.20.5 (protocol 766); 1.20.1..1.20.4 always authenticate.
        let should_authenticate = if r.remaining() > 0 {
            <Bool as crate::buf::Codec<bool>>::read(r)?
        } else {
            true
        };
        Ok(Self {
            server_id,
            public_key,
            challenge,
            should_authenticate,
        })
    }
    fn write(v: &Self, w: &mut crate::buf::Writer) {
        <StrN<20> as crate::buf::Codec<String>>::write(&v.server_id, w);
        <Bytes as crate::buf::Codec<Vec<u8>>>::write(&v.public_key, w);
        <Bytes as crate::buf::Codec<Vec<u8>>>::write(&v.challenge, w);
        <Bool as crate::buf::Codec<bool>>::write(&v.should_authenticate, w);
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct LoginFinished {
    pub profile: GameProfile,
    pub session: u128,
}
impl crate::buf::Codec<LoginFinished> for LoginFinished {
    fn read(r: &mut crate::buf::Reader) -> crate::Result<Self> {
        let profile = <GameProfile as crate::buf::Codec<GameProfile>>::read(r)?;
        // 775..777 (26.x) append a 16-byte `session: Uuid`.
        // 766..767 (1.20.5..1.21.1) append a 1-byte `strict_error_handling: bool`.
        // 763..765 (1.20.1..1.20.4) and 768..774 (1.21.2..1.21.11) end right after `GameProfile`.
        let session = if r.remaining() >= 16 {
            <Uuid as crate::buf::Codec<u128>>::read(r)?
        } else {
            if r.remaining() == 1 {
                let _ = r.bool()?;
            }
            0
        };
        Ok(Self { profile, session })
    }
    fn write(v: &Self, w: &mut crate::buf::Writer) {
        <GameProfile as crate::buf::Codec<GameProfile>>::write(&v.profile, w);
        <Uuid as crate::buf::Codec<u128>>::write(&v.session, w);
    }
}
packet!(Compression { threshold: i32 => VarInt });
packet!(CustomQuery { transaction: i32 => VarInt, channel: String => Ident, data: Vec<u8> => Rest });
packet!(CookieRequest { key: String => Ident });

packet_enum!(Clientbound, crate::ids::login::clientbound {
    Disconnect(Disconnect) = LOGIN_DISCONNECT,
    Hello(Hello) = HELLO,
    LoginFinished(LoginFinished) = LOGIN_FINISHED,
    Compression(Compression) = LOGIN_COMPRESSION,
    CustomQuery(CustomQuery) = CUSTOM_QUERY,
    CookieRequest(CookieRequest) = COOKIE_REQUEST,
});

packet!(ServerHello { name: String => StrN<16>, uuid: u128 => Uuid });
packet!(Key { key: Vec<u8> => Bytes, challenge: Vec<u8> => Bytes });
packet!(CustomQueryAnswer { transaction: i32 => VarInt, payload: Option<Vec<u8>> => Opt<Rest> });
packet!(CookieResponse { key: String => Ident, payload: Option<Vec<u8>> => Opt<Bytes> });

packet_enum!(Serverbound, crate::ids::login::serverbound {
    Hello(ServerHello) = HELLO,
    Key(Key) = KEY,
    CustomQueryAnswer(CustomQueryAnswer) = CUSTOM_QUERY_ANSWER,
    LoginAcknowledged(Empty) = LOGIN_ACKNOWLEDGED,
    CookieResponse(CookieResponse) = COOKIE_RESPONSE,
});

#[cfg(test)]
mod tests {
    use super::*;
    use crate::packets::round_trip;

    #[test]
    fn login_round_trips() {
        let profile = GameProfile {
            uuid: crate::crypto::offline_uuid("Horizon"),
            name: "Horizon".into(),
            properties: vec![Property {
                name: "textures".into(),
                value: "e30=".into(),
                signature: None,
            }],
        };
        for p in [
            Clientbound::Disconnect(Disconnect {
                reason_json: "{\"text\":\"bye\"}".into(),
            }),
            Clientbound::Hello(Hello {
                server_id: String::new(),
                public_key: vec![1, 2],
                challenge: vec![3; 4],
                should_authenticate: true,
            }),
            Clientbound::LoginFinished(LoginFinished {
                profile,
                session: 7,
            }),
            Clientbound::Compression(Compression { threshold: 256 }),
            Clientbound::CustomQuery(CustomQuery {
                transaction: 3,
                channel: "velocity:player_info".into(),
                data: vec![1],
            }),
            Clientbound::CookieRequest(CookieRequest { key: "a:b".into() }),
        ] {
            round_trip(p, Clientbound::encode, Clientbound::decode);
        }
        for p in [
            Serverbound::Hello(ServerHello {
                name: "Horizon".into(),
                uuid: 1,
            }),
            Serverbound::Key(Key {
                key: vec![9; 128],
                challenge: vec![8; 128],
            }),
            Serverbound::CustomQueryAnswer(CustomQueryAnswer {
                transaction: 3,
                payload: None,
            }),
            Serverbound::LoginAcknowledged(Empty),
            Serverbound::CookieResponse(CookieResponse {
                key: "a:b".into(),
                payload: Some(vec![1]),
            }),
        ] {
            round_trip(p, Serverbound::encode, Serverbound::decode);
        }
    }

    #[test]
    fn rejects_trailing_bytes_and_long_names() {
        let mut body = Serverbound::LoginAcknowledged(Empty).encode();
        body.push(0);
        assert!(Serverbound::decode(3, &body[1..]).is_err());
        let long = Serverbound::Hello(ServerHello {
            name: "x".repeat(17),
            uuid: 0,
        })
        .encode();
        assert!(Serverbound::decode(0, &long[1..]).is_err());
    }
}
