# Punchy Rust voor Horizon — onvolledige preview

Dit is **geen volledige port van Punchy 3.1b**. De originele JAR is geïnventariseerd met REA, alle 894 Java-klassen zijn geïmporteerd/geanalyseerd met Ghidra en de Java-code is lokaal gedecompileerd. Dat bewijst de statische dekking, niet dat ieder gedrag al begrepen of gereproduceerd is.

## Wat er nu staat

- Een zelfstandige Rust-animatiekern: clipformaat, expressies, 30 easingcurves, onafhankelijke kanalen, legacy/v2-coördinaten, pre/post-rotaties, interpolatie, effecten, geluidscursor en onderbroken crossfades.
- Actiecoördinatie en een gedeeltelijke implementatie van attack/mining/equip/inspect/bow/crossbow/shield/trident/spear/eat/drink.
- Parsing van Punchy's tuningtemplate, handed clipselectie, combo's en een deel van item/group-configuratie.
- Een revisiegepinde Horizon-adapter met de herstelde camera/body/arm-hiërarchie, griptransformaties, standaard itemaanhechting en third-person itemdisplaycontext. De volledige procedurele en speciale renderlagen ontbreken nog. De eerdere Horizon-poseadapter blijft als vergelijkingsoptie beschikbaar.
- 223 clips, de skelethiërarchie en de standaardtuning worden met de Rust-game meegecompileerd. Tijdens het spelen zijn geen Punchy-JAR, Java, Fabric of losse Punchy-configuratiebestanden nodig.

## Grenzen

De volledige editor, profiel- en packmixing, alle samengestelde handovergangen, locomotion, derde persoon, itemmodelsystemen, physics, overlays, particles en compatibiliteitslagen ontbreken of zijn slechts gedeeltelijk onderzocht/geport. De consume/mining-v2-fasecontrollers zijn nu gekoppeld aan de client: start, herhaling en einde zijn in een tijdelijke spelwereld getest voor eten, drinken en mijnen. Complexe equip-/offhandsnapshots en alle onderbrekingscombinaties vereisen nog aanvullende validatie. De geluidscursor is getest; de adapter speelt de oorspronkelijke animatiegeluiden nog niet af. De adapter bewaart nu doorlopende uitgaande animaties bij onderbrekingen en mengt de lokale bottransformaties vóór de aanhechtingsketen. Volledige overeenkomst met alle PoseHandler-overlap/rebasevarianten is nog niet bewezen. De UI toont alleen previewinstellingen. De afspeelsnelheid volgt de expliciete equip-/consume-/timeline-regels uit PoseHandler; eenvoudige numerieke tests dekken de grensgevallen.

De standaardrenderer gebruikt nu de herstelde skelethiërarchie. 128 matrixvergelijkingen met de oorspronkelijke Java-klasse controleren de transformaties, inclusief legacygrips met shear. Dit bewijst nog niet de volledige zichtbare gamepipeline: volledige armownership, procedurele beweging en alle speciale itemrenderers ontbreken. Het pakket heet daarom bewust `punchy-rust-preview-0.1.2.hnmod`.

## Correcties in 0.1.2

De normale client gebruikt nu `Game.mining` voor mijnanimaties. Eten/drinken/mijnen spelen hun geselecteerde start-loop-end-reeks af. Ingebouwde clipherhaling wordt gerespecteerd, terwijl de expressieklok doorloopt. Succesvolle item-, blok- en entity-interacties leveren gebruikstriggers aan. De tweede arm volgt de tijdvensters voor dual-handed/lege-offhand-zichtbaarheid; camera/body-beweging blijft gedeeld bij een enkelarmige actie.

## Revisies

- Game: Minecraft 26.3.
- Horizon: `81ae4a7d8f27d38e5cabbdcda0a06b2c4b000045`.
- horizon-mixins: `ddd9700b21997c95d95ac6ee5ecafe3830cf622a`.
- Geanalyseerde JAR SHA-256: `422533a6f412c9f2f2c421f1a5e26e42825eebdf904b678c44e3f3230187de25`.

Dit verschilt van de oudere pins van Flowing Capes en Replay in deze werkmap. Hun gezamenlijke werking met deze preview is niet gevalideerd.

## Lokale preview gebruiken

Het zelfstandige native pakket staat in `dist/punchy-rust-preview-0.1.2.hnmod` vanaf de werkmap. Het bevat Rust-broncode, de benodigde animatie-/model-/tuningdata en de oorspronkelijke licentievermeldingen. Het bevat geen Java-klassen en gebruikt geen JAR tijdens het spelen.

Installeer dit ene `.hnmod`-pakket in de bedoelde Horizon-instance en laat de launcher de native build maken. Dit is de normale bouwstap voor Horizon-native mods; er zijn geen aanvullende bestanden of runtimes nodig. De gegevens worden in de gamebinary ingebouwd, dus er hoeft ook geen losse datamap naast de game te blijven staan.

Android Options → Mods gebruikt de Punchy-modrij als enige ON/OFF-schakelaar; de instellingenpagina bevat snelheid, reload en de keuze tussen Punchy- en Horizon-handposes. F6 inspecteert; F7 herlaadt de ingebouwde animaties. Beide toetsen zijn in de modinstellingen te wijzigen.

De animatiegegevens zijn afkomstig uit het voor deze lokale port aangeleverde Punchy 3.1b-bestand; `LICENSE-Punchy` bewaart de oorspronkelijke vermelding. De rendereradapter bevat Horizon MIT-code; zie `LICENSE-Horizon`.

## Reproduceren

Vanaf de werkmap:

```sh
cargo test --manifest-path native/punchy/Cargo.toml --offline
cargo clippy --manifest-path native/punchy/Cargo.toml --offline --all-targets -- -D warnings
python3 native/punchy/tools/check_math_parity.py
python3 native/punchy/tools/check_matrix_parity.py
cargo run --manifest-path native/punchy/Cargo.toml --offline --example inspect -- MODS/newmods/punchy-3.1b-fabric-26.3.jar
python3.11 native/punchy/tools/package.py
```

De Java-oracle voert alleen oorspronkelijke wiskunde-/expressie-/geluidscursorklassen uit. Vereist Java 25 en de lokale JOML-library; hun paden zijn via `PUNCHY_JAVA_HOME` en `PUNCHY_JOML_JAR` instelbaar. De Java-vergelijkingen en enkele ontwikkeltests gebruiken de oorspronkelijke JAR of lokale analyse-output. Dit zijn ontwikkelhulpmiddelen; de geïnstalleerde native mod gebruikt uitsluitend zijn ingebouwde gegevens.

De native compilecontrole gebruikt een aparte export in `build/punchy-source` en een gegenereerde stage `build/punchy-stage`. Upstream-bron en echte spelerdata zijn niet aangepast. Op macOS moet `horizon-mixins apply` naar een nieuwe outputmap schrijven; atomische vervanging van een bestaande stage wordt door deze toolversie geweigerd.

De grafische smoke-test staat in `tools/runtime_smoke.py`; deze gebruikt geen Punchy-JAR en start met nieuwe tijdelijke game-/launcher-/XDG-mappen en bewaart zijn checks en screenshots in de gekozen rapportmap. Zie `reports/punchy/coverage.md`, `runtime-smoke.json`, `rust-tests.log` en `math-parity.log` voor de uitgevoerde controles en hun grenzen.
