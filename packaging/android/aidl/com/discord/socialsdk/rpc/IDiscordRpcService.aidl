// The Discord Android app's Rich Presence service, as its Social SDK uses it internally.
// Interface reverse-engineered by Nightdavisao (antisocialcord-kt, MIT); see
// THIRD_PARTY_NOTICES.md. Volt's use: dev/horizon/android/VoltDiscordRpc.java.
package com.discord.socialsdk.rpc;

import com.discord.socialsdk.rpc.IDiscordRpcCallback;
import com.discord.socialsdk.rpc.IDiscordRpcConnection;

interface IDiscordRpcService {
    IDiscordRpcConnection connect(long applicationId, String version, IDiscordRpcCallback callback);
}
