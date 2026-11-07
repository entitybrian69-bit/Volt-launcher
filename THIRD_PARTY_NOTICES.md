# Third-party notices

## wgpu-hal

`third_party/wgpu-hal` is [wgpu-hal](https://github.com/gfx-rs/wgpu) 30.0.1 by the gfx-rs
developers, with upstream fixes applied
([gfx-rs/wgpu#10124](https://github.com/gfx-rs/wgpu/pull/10124),
[gfx-rs/wgpu#10714](https://github.com/gfx-rs/wgpu/pull/10714)). It is licensed under MIT or
Apache-2.0; both licenses are in that folder (`LICENSE.MIT`, `LICENSE.APACHE`).

## Pixelify Sans

`crates/horizon-theme/fonts/PixelifySans-*.ttf` is [Pixelify Sans](https://github.com/eifetx/Pixelify-Sans)
by The Pixelify Sans Project Authors, licensed under the SIL Open Font License 1.1
(`crates/horizon-theme/fonts/OFL-PixelifySans.txt`).

## Pixelarticons

`crates/horizon-theme/icons/pixel/*.svg` are icons from
[Pixelarticons](https://github.com/halfmage/pixelarticons) by Gerrit Halfmann, under the MIT
license (`crates/horizon-theme/icons/pixel/LICENSE-pixelarticons.txt`). `star.svg` is Horizon's own, drawn on
the same grid.

## winit

`third_party/winit` is [winit](https://github.com/rust-windowing/winit) 0.30.13,
licensed under Apache-2.0 (see `third_party/winit/LICENSE`). The Android backend
has local mouse and modifier-event fixes described in `third_party/winit/PATCHES.md`.

## AMD FidelityFX SDK (FSR 3.1 frame generation)

`crates/horizon-client-render/src/gfx/frame_gen/optical_flow.wgsl` is a port to WGSL of the
optical flow shaders of the [AMD FidelityFX SDK](https://github.com/GPUOpen-LibrariesAndSDKs/FidelityFX-SDK)
1.1.4 (`sdk/include/FidelityFX/gpu/opticalflow`, at commit
[c6efa6bf7f20](https://github.com/GPUOpen-LibrariesAndSDKs/FidelityFX-SDK/tree/c6efa6bf7f2027b3ec94f28578bb5965eabb9e55)),
and `optical_flow.rs` follows the SDK's `sdk/src/components/opticalflow/ffx_opticalflow.cpp`
(its passes, their order and their dispatch sizes). Copyright (C) 2024 Advanced Micro Devices,
Inc., under the MIT license, whose text heads both files. AMD and FidelityFX are AMD's
trademarks; Horizon is not affiliated with AMD.

## Mod loader logos

The launcher shows the logo of the mod loader an instance plays with.

- `launcher/assets/logos/quilt.png` is the Quilt mini icon (`brand/128png/quilt_mini_icon_transparent.png`)
  from [QuiltMC/art](https://github.com/QuiltMC/art), by the Quilt Project, dedicated to the public
  domain under [CC0 1.0](https://creativecommons.org/publicdomain/zero/1.0/).
- `launcher/assets/logos/neoforge.png` is NeoForged's icon (`art/neoforged_icon_16x16.png`) from
  [neoforged/.github](https://github.com/neoforged/.github/tree/main/art), © the NeoForged team,
  created by Ridanisaurus, under [CC BY 4.0](https://creativecommons.org/licenses/by/4.0/). It is
  unmodified (the launcher scales it up by whole pixels).
- Fabric's and Forge's icons are not shipped: neither publishes a license for its logo. The launcher
  fetches them from their own sites when an instance needs them
  ([fabric-loader's icon](https://github.com/FabricMC/fabric-loader/blob/master/src/main/resources/assets/fabricloader/icon.png),
  [files.minecraftforge.net's](https://files.minecraftforge.net/)).

Each logo belongs to its project. Showing it says which loader an instance runs; it does not mean
the project endorses Horizon.

## Sign in with Microsoft

The launcher's sign-in buttons follow Microsoft's published branding guidelines for apps that sign in
with Microsoft accounts
([Sign in with Microsoft branding](https://learn.microsoft.com/en-us/entra/identity-platform/howto-add-branding-in-apps)).
The four-square Microsoft symbol in `launcher/src/microsoft.rs` reproduces the squares, positions and
colours of Microsoft's official `ms-symbollockup_mssymbol_19.svg` from that page, unmodified. Microsoft
and its logo are trademarks of the Microsoft group of companies; Horizon is not endorsed by Microsoft.

## Upstream mod icons

The following icons are derived from the listed original project assets, resized
or resampled to 64 × 64 pixels. Their use does not imply endorsement; Volt's
native features are not the original Java mods.

- AppleSkin, `resources/assets/appleskin/appleskin.png`,
  [squeek502/AppleSkin](https://github.com/squeek502/AppleSkin/tree/66fcdb8b12820c9b1cf3147abfdcf5530559f62b), Unlicense.
- Armor HUD, `src/main/resources/assets/armor_hud/logo/ArmorHUDLogo.png`,
  [SaolGhra/Armor-Hud](https://github.com/SaolGhra/Armor-Hud/tree/8f51be5289fa9f667bc352101b299c6ba710f998), MIT.
- Low Fire, `src/main/resources/assets/lowfire/icon.png`,
  [UltimateBoomer/mc-lowfire](https://github.com/UltimateBoomer/mc-lowfire/tree/c60755af265f5e70d094aa7d1537cf2ccf69acef), MIT.
- Mod Menu, `src/main/resources/assets/modmenu/icon.png`,
  [TerraformersMC/ModMenu](https://github.com/TerraformersMC/ModMenu/tree/4d73e51f6e3614dc35fd84107a9ba894a8934a2b), MIT.
- ShulkerBoxTooltip, `fabric/src/main/resources/assets/shulkerboxtooltip/icon.png`,
  [MisterPeModder/ShulkerBoxTooltip](https://github.com/MisterPeModder/ShulkerBoxTooltip/tree/68e7f8e280f13f59a18a95f378b4f57b606db5fa), MIT.
- Language Reload, `src/main/resources/assets/languagereload/icon.png`,
  [Jerozgen/LanguageReload](https://github.com/Jerozgen/LanguageReload/tree/b92a6fb9b7a29fa397d1af037b88bf2519ba531e), MIT.
- FastQuit, `src/main/resources/assets/fastquit/icon.png`,
  [KingContaria/FastQuit](https://github.com/KingContaria/FastQuit/tree/8efed17360a89ca0b23d56a88830a2b3d535eeed), MIT.

License texts for these assets follow:

### appleskin

```text
This is free and unencumbered software released into the public domain.

Anyone is free to copy, modify, publish, use, compile, sell, or
distribute this software, either in source code form or as a compiled
binary, for any purpose, commercial or non-commercial, and by any
means.

In jurisdictions that recognize copyright laws, the author or authors
of this software dedicate any and all copyright interest in the
software to the public domain. We make this dedication for the benefit
of the public at large and to the detriment of our heirs and
successors. We intend this dedication to be an overt act of
relinquishment in perpetuity of all present and future rights to this
software under copyright law.

THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF ANY KIND,
EXPRESS OR IMPLIED, INCLUDING BUT NOT LIMITED TO THE WARRANTIES OF
MERCHANTABILITY, FITNESS FOR A PARTICULAR PURPOSE AND NONINFRINGEMENT.
IN NO EVENT SHALL THE AUTHORS BE LIABLE FOR ANY CLAIM, DAMAGES OR
OTHER LIABILITY, WHETHER IN AN ACTION OF CONTRACT, TORT OR OTHERWISE,
ARISING FROM, OUT OF OR IN CONNECTION WITH THE SOFTWARE OR THE USE OR
OTHER DEALINGS IN THE SOFTWARE.

For more information, please refer to <http://unlicense.org>

```

### armor_hud

```text
The MIT License (MIT)

Copyright (c) 2024 SaolGhra

Permission is hereby granted, free of charge, to any person obtaining a copy
of this software and associated documentation files (the "Software"), to deal
in the Software without restriction, including without limitation the rights
to use, copy, modify, merge, publish, distribute, sublicense, and/or sell
copies of the Software, and to permit persons to whom the Software is
furnished to do so, subject to the following conditions:

The above copyright notice and this permission notice shall be included in
all copies or substantial portions of the Software.

THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF ANY KIND, EXPRESS OR
IMPLIED, INCLUDING BUT NOT LIMITED TO THE WARRANTIES OF MERCHANTABILITY,
FITNESS FOR A PARTICULAR PURPOSE AND NONINFRINGEMENT. IN NO EVENT SHALL THE
AUTHORS OR COPYRIGHT HOLDERS BE LIABLE FOR ANY CLAIM, DAMAGES OR OTHER
LIABILITY, WHETHER IN AN ACTION OF CONTRACT, TORT OR OTHERWISE, ARISING FROM,
OUT OF OR IN CONNECTION WITH THE SOFTWARE OR THE USE OR OTHER DEALINGS IN
THE SOFTWARE.

```

### low_fire

```text
Copyright 2021 Steven Cao

Permission is hereby granted, free of charge, to any person obtaining a copy of this software and associated documentation files (the "Software"), to deal in the Software without restriction, including without limitation the rights to use, copy, modify, merge, publish, distribute, sublicense, and/or sell copies of the Software, and to permit persons to whom the Software is furnished to do so, subject to the following conditions:

The above copyright notice and this permission notice shall be included in all copies or substantial portions of the Software.

THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF ANY KIND, EXPRESS OR IMPLIED, INCLUDING BUT NOT LIMITED TO THE WARRANTIES OF MERCHANTABILITY, FITNESS FOR A PARTICULAR PURPOSE AND NONINFRINGEMENT. IN NO EVENT SHALL THE AUTHORS OR COPYRIGHT HOLDERS BE LIABLE FOR ANY CLAIM, DAMAGES OR OTHER LIABILITY, WHETHER IN AN ACTION OF CONTRACT, TORT OR OTHERWISE, ARISING FROM, OUT OF OR IN CONNECTION WITH THE SOFTWARE OR THE USE OR OTHER DEALINGS IN THE SOFTWARE.
```

### mod_menu

```text
MIT License

Copyright (c) 2018-2020 Prospector

Permission is hereby granted, free of charge, to any person obtaining a copy
of this software and associated documentation files (the "Software"), to deal
in the Software without restriction, including without limitation the rights
to use, copy, modify, merge, publish, distribute, sublicense, and/or sell
copies of the Software, and to permit persons to whom the Software is
furnished to do so, subject to the following conditions:

The above copyright notice and this permission notice shall be included in all
copies or substantial portions of the Software.

THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF ANY KIND, EXPRESS OR
IMPLIED, INCLUDING BUT NOT LIMITED TO THE WARRANTIES OF MERCHANTABILITY,
FITNESS FOR A PARTICULAR PURPOSE AND NONINFRINGEMENT. IN NO EVENT SHALL THE
AUTHORS OR COPYRIGHT HOLDERS BE LIABLE FOR ANY CLAIM, DAMAGES OR OTHER
LIABILITY, WHETHER IN AN ACTION OF CONTRACT, TORT OR OTHERWISE, ARISING FROM,
OUT OF OR IN CONNECTION WITH THE SOFTWARE OR THE USE OR OTHER DEALINGS IN THE
SOFTWARE.

```

### shulkerboxtooltip

```text
MIT License

Copyright (c) 2019 Yanis Guaye

Permission is hereby granted, free of charge, to any person obtaining a copy
of this software and associated documentation files (the "Software"), to deal
in the Software without restriction, including without limitation the rights
to use, copy, modify, merge, publish, distribute, sublicense, and/or sell
copies of the Software, and to permit persons to whom the Software is
furnished to do so, subject to the following conditions:

The above copyright notice and this permission notice shall be included in all
copies or substantial portions of the Software.

THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF ANY KIND, EXPRESS OR
IMPLIED, INCLUDING BUT NOT LIMITED TO THE WARRANTIES OF MERCHANTABILITY,
FITNESS FOR A PARTICULAR PURPOSE AND NONINFRINGEMENT. IN NO EVENT SHALL THE
AUTHORS OR COPYRIGHT HOLDERS BE LIABLE FOR ANY CLAIM, DAMAGES OR OTHER
LIABILITY, WHETHER IN AN ACTION OF CONTRACT, TORT OR OTHERWISE, ARISING FROM,
OUT OF OR IN CONNECTION WITH THE SOFTWARE OR THE USE OR OTHER DEALINGS IN THE
SOFTWARE.

```

### language_reload

```text
The MIT License (MIT)

Copyright (c) 2021 Jerozgen

Permission is hereby granted, free of charge, to any person obtaining a copy
of this software and associated documentation files (the "Software"), to deal
in the Software without restriction, including without limitation the rights
to use, copy, modify, merge, publish, distribute, sublicense, and/or sell
copies of the Software, and to permit persons to whom the Software is
furnished to do so, subject to the following conditions:

The above copyright notice and this permission notice shall be included in
all copies or substantial portions of the Software.

THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF ANY KIND, EXPRESS OR
IMPLIED, INCLUDING BUT NOT LIMITED TO THE WARRANTIES OF MERCHANTABILITY,
FITNESS FOR A PARTICULAR PURPOSE AND NONINFRINGEMENT. IN NO EVENT SHALL THE
AUTHORS OR COPYRIGHT HOLDERS BE LIABLE FOR ANY CLAIM, DAMAGES OR OTHER
LIABILITY, WHETHER IN AN ACTION OF CONTRACT, TORT OR OTHERWISE, ARISING FROM,
OUT OF OR IN CONNECTION WITH THE SOFTWARE OR THE USE OR OTHER DEALINGS IN
THE SOFTWARE.

```

### fast_quit

```text
The MIT License (MIT)

Copyright (c) 2022

Permission is hereby granted, free of charge, to any person obtaining a copy
of this software and associated documentation files (the "Software"), to deal
in the Software without restriction, including without limitation the rights
to use, copy, modify, merge, publish, distribute, sublicense, and/or sell
copies of the Software, and to permit persons to whom the Software is
furnished to do so, subject to the following conditions:

The above copyright notice and this permission notice shall be included in
all copies or substantial portions of the Software.

THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF ANY KIND, EXPRESS OR
IMPLIED, INCLUDING BUT NOT LIMITED TO THE WARRANTIES OF MERCHANTABILITY,
FITNESS FOR A PARTICULAR PURPOSE AND NONINFRINGEMENT. IN NO EVENT SHALL THE
AUTHORS OR COPYRIGHT HOLDERS BE LIABLE FOR ANY CLAIM, DAMAGES OR OTHER
LIABILITY, WHETHER IN AN ACTION OF CONTRACT, TORT OR OTHERWISE, ARISING FROM,
OUT OF OR IN CONNECTION WITH THE SOFTWARE OR THE USE OR OTHER DEALINGS IN
THE SOFTWARE.
```

### betterf3 icon

Original asset: [`fabric/src/main/resources/icon.png`](https://github.com/cominixo/BetterF3/blob/e93813eb3001368393bf22e0e2884782fc7d8b2d/fabric/src/main/resources/icon.png), resampled to 64 × 64. License: MIT.

```text
MIT License

Copyright (c) 2020 - 2022 cominixo / TreyRuffy

Permission is hereby granted, free of charge, to any person obtaining a copy
of this software and associated documentation files (the "Software"), to deal
in the Software without restriction, including without limitation the rights
to use, copy, modify, merge, publish, distribute, sublicense, and/or sell
copies of the Software, and to permit persons to whom the Software is
furnished to do so, subject to the following conditions:

The above copyright notice and this permission notice shall be included in all
copies or substantial portions of the Software.

THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF ANY KIND, EXPRESS OR
IMPLIED, INCLUDING BUT NOT LIMITED TO THE WARRANTIES OF MERCHANTABILITY,
FITNESS FOR A PARTICULAR PURPOSE AND NONINFRINGEMENT. IN NO EVENT SHALL THE
AUTHORS OR COPYRIGHT HOLDERS BE LIABLE FOR ANY CLAIM, DAMAGES OR OTHER
LIABILITY, WHETHER IN AN ACTION OF CONTRACT, TORT OR OTHERWISE, ARISING FROM,
OUT OF OR IN CONNECTION WITH THE SOFTWARE OR THE USE OR OTHER DEALINGS IN THE
SOFTWARE.
```

### better_ping_display icon

Original asset: [`src/main/resources/assets/betterpingdisplay/icon.png`](https://github.com/vladmarica/better-ping-display-fabric/blob/96ddda664ff4c4eb2e119c22d3ed6e93b1012294/src/main/resources/assets/betterpingdisplay/icon.png), resampled to 64 × 64. License: MIT.

```text
MIT License

Copyright (c) 2020 Vlad Marica

Permission is hereby granted, free of charge, to any person obtaining a copy
of this software and associated documentation files (the "Software"), to deal
in the Software without restriction, including without limitation the rights
to use, copy, modify, merge, publish, distribute, sublicense, and/or sell
copies of the Software, and to permit persons to whom the Software is
furnished to do so, subject to the following conditions:

The above copyright notice and this permission notice shall be included in all
copies or substantial portions of the Software.

THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF ANY KIND, EXPRESS OR
IMPLIED, INCLUDING BUT NOT LIMITED TO THE WARRANTIES OF MERCHANTABILITY,
FITNESS FOR A PARTICULAR PURPOSE AND NONINFRINGEMENT. IN NO EVENT SHALL THE
AUTHORS OR COPYRIGHT HOLDERS BE LIABLE FOR ANY CLAIM, DAMAGES OR OTHER
LIABILITY, WHETHER IN AN ACTION OF CONTRACT, TORT OR OTHERWISE, ARISING FROM,
OUT OF OR IN CONNECTION WITH THE SOFTWARE OR THE USE OR OTHER DEALINGS IN THE
SOFTWARE.
```

### controlling icon

Original asset: [`common/src/main/resources/icon.png`](https://github.com/jaredlll08/Controlling/blob/9cf667ac622a22496b71caa9a625c0f17eed643f/common/src/main/resources/icon.png), resampled to 64 × 64. License: MIT.

```text
MIT License

Copyright (c) 2021 Jared

Permission is hereby granted, free of charge, to any person obtaining a copy
of this software and associated documentation files (the "Software"), to deal
in the Software without restriction, including without limitation the rights
to use, copy, modify, merge, publish, distribute, sublicense, and/or sell
copies of the Software, and to permit persons to whom the Software is
furnished to do so, subject to the following conditions:

The above copyright notice and this permission notice shall be included in all
copies or substantial portions of the Software.

THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF ANY KIND, EXPRESS OR
IMPLIED, INCLUDING BUT NOT LIMITED TO THE WARRANTIES OF MERCHANTABILITY,
FITNESS FOR A PARTICULAR PURPOSE AND NONINFRINGEMENT. IN NO EVENT SHALL THE
AUTHORS OR COPYRIGHT HOLDERS BE LIABLE FOR ANY CLAIM, DAMAGES OR OTHER
LIABILITY, WHETHER IN AN ACTION OF CONTRACT, TORT OR OTHERWISE, ARISING FROM,
OUT OF OR IN CONNECTION WITH THE SOFTWARE OR THE USE OR OTHER DEALINGS IN THE
SOFTWARE.
```

### mouse_tweaks icon

Original asset: [`src/main/resources/mousetweaks_logo.png`](https://github.com/YaLTeR/MouseTweaks/blob/e8ecd08ed94192f6f19d3cbb630707c8670a4d66/src/main/resources/mousetweaks_logo.png), resampled to 64 × 64. License: BSD-3-Clause.

```text
BSD 3-Clause License

Copyright (c) 2023, Ivan Molodetskikh

Redistribution and use in source and binary forms, with or without
modification, are permitted provided that the following conditions are met:

1. Redistributions of source code must retain the above copyright notice, this
   list of conditions and the following disclaimer.

2. Redistributions in binary form must reproduce the above copyright notice,
   this list of conditions and the following disclaimer in the documentation
   and/or other materials provided with the distribution.

3. Neither the name of the copyright holder nor the names of its
   contributors may be used to endorse or promote products derived from
   this software without specific prior written permission.

THIS SOFTWARE IS PROVIDED BY THE COPYRIGHT HOLDERS AND CONTRIBUTORS "AS IS"
AND ANY EXPRESS OR IMPLIED WARRANTIES, INCLUDING, BUT NOT LIMITED TO, THE
IMPLIED WARRANTIES OF MERCHANTABILITY AND FITNESS FOR A PARTICULAR PURPOSE ARE
DISCLAIMED. IN NO EVENT SHALL THE COPYRIGHT HOLDER OR CONTRIBUTORS BE LIABLE
FOR ANY DIRECT, INDIRECT, INCIDENTAL, SPECIAL, EXEMPLARY, OR CONSEQUENTIAL
DAMAGES (INCLUDING, BUT NOT LIMITED TO, PROCUREMENT OF SUBSTITUTE GOODS OR
SERVICES; LOSS OF USE, DATA, OR PROFITS; OR BUSINESS INTERRUPTION) HOWEVER
CAUSED AND ON ANY THEORY OF LIABILITY, WHETHER IN CONTRACT, STRICT LIABILITY,
OR TORT (INCLUDING NEGLIGENCE OR OTHERWISE) ARISING IN ANY WAY OUT OF THE USE
OF THIS SOFTWARE, EVEN IF ADVISED OF THE POSSIBILITY OF SUCH DAMAGE.
```

### no_fog icon

Original asset: [`src/main/resources/assets/no_fog/icon.png`](https://github.com/Virtuoel/NoFog/blob/65233d9266fbef0272bf0b7bbc031226a7f26fa1/src/main/resources/assets/no_fog/icon.png), resampled to 64 × 64. License: MIT.

```text
MIT License

Copyright (c) 2020 - 2024 Virtuoel

Permission is hereby granted, free of charge, to any person obtaining a copy
of this software and associated documentation files (the "Software"), to deal
in the Software without restriction, including without limitation the rights
to use, copy, modify, merge, publish, distribute, sublicense, and/or sell
copies of the Software, and to permit persons to whom the Software is
furnished to do so, subject to the following conditions:

The above copyright notice and this permission notice shall be included in all
copies or substantial portions of the Software.

THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF ANY KIND, EXPRESS OR
IMPLIED, INCLUDING BUT NOT LIMITED TO THE WARRANTIES OF MERCHANTABILITY,
FITNESS FOR A PARTICULAR PURPOSE AND NONINFRINGEMENT. IN NO EVENT SHALL THE
AUTHORS OR COPYRIGHT HOLDERS BE LIABLE FOR ANY CLAIM, DAMAGES OR OTHER
LIABILITY, WHETHER IN AN ACTION OF CONTRACT, TORT OR OTHERWISE, ARISING FROM,
OUT OF OR IN CONNECTION WITH THE SOFTWARE OR THE USE OR OTHER DEALINGS IN THE
SOFTWARE.
```

### ferritecore icon

Original asset: [`Common/src/main/resources/logo.png`](https://github.com/malte0811/FerriteCore/blob/0cef1f2add1f1329aa6e690e8e292acd625c5c6d/Common/src/main/resources/logo.png), resampled to 64 × 64. License: MIT.

```text
MIT License

Copyright (c) 2020 malte0811

Permission is hereby granted, free of charge, to any person obtaining a copy
of this software and associated documentation files (the "Software"), to deal
in the Software without restriction, including without limitation the rights
to use, copy, modify, merge, publish, distribute, sublicense, and/or sell
copies of the Software, and to permit persons to whom the Software is
furnished to do so, subject to the following conditions:

The above copyright notice and this permission notice shall be included in all
copies or substantial portions of the Software.

THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF ANY KIND, EXPRESS OR
IMPLIED, INCLUDING BUT NOT LIMITED TO THE WARRANTIES OF MERCHANTABILITY,
FITNESS FOR A PARTICULAR PURPOSE AND NONINFRINGEMENT. IN NO EVENT SHALL THE
AUTHORS OR COPYRIGHT HOLDERS BE LIABLE FOR ANY CLAIM, DAMAGES OR OTHER
LIABILITY, WHETHER IN AN ACTION OF CONTRACT, TORT OR OTHERWISE, ARISING FROM,
OUT OF OR IN CONNECTION WITH THE SOFTWARE OR THE USE OR OTHER DEALINGS IN THE
SOFTWARE.
```

### dynamic_fps icon

Original asset: [`platforms/common/src/main/resources/assets/dynamic_fps/textures/icon.png`](https://github.com/juliand665/Dynamic-FPS/blob/fa09373b4112c324468f32177dbaeafb5f469278/platforms/common/src/main/resources/assets/dynamic_fps/textures/icon.png), resampled to 64 × 64. License: MIT.

```text
MIT License

Copyright (c) 2019 - 2024 Julian Dunskus

Permission is hereby granted, free of charge, to any person obtaining a copy
of this software and associated documentation files (the "Software"), to deal
in the Software without restriction, including without limitation the rights
to use, copy, modify, merge, publish, distribute, sublicense, and/or sell
copies of the Software, and to permit persons to whom the Software is
furnished to do so, subject to the following conditions:

The above copyright notice and this permission notice shall be included in all
copies or substantial portions of the Software.

THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF ANY KIND, EXPRESS OR
IMPLIED, INCLUDING BUT NOT LIMITED TO THE WARRANTIES OF MERCHANTABILITY,
FITNESS FOR A PARTICULAR PURPOSE AND NONINFRINGEMENT. IN NO EVENT SHALL THE
AUTHORS OR COPYRIGHT HOLDERS BE LIABLE FOR ANY CLAIM, DAMAGES OR OTHER
LIABILITY, WHETHER IN AN ACTION OF CONTRACT, TORT OR OTHERWISE, ARISING FROM,
OUT OF OR IN CONNECTION WITH THE SOFTWARE OR THE USE OR OTHER DEALINGS IN THE
SOFTWARE.
```

### bad_optimizations icon

Original asset: [`fabric/src/main/resources/assets/badoptimizations/icon.png`](https://github.com/imthosea/BadOptimizations/blob/5de4a3ad4299909178d8995dc0bc80626be48d44/fabric/src/main/resources/assets/badoptimizations/icon.png), resampled to 64 × 64. License: MIT.

```text
The MIT License (MIT)

Copyright (c) 2025 Thosea

Permission is hereby granted, free of charge, to any person obtaining a copy
of this software and associated documentation files (the "Software"), to deal
in the Software without restriction, including without limitation the rights
to use, copy, modify, merge, publish, distribute, sublicense, and/or sell
copies of the Software, and to permit persons to whom the Software is
furnished to do so, subject to the following conditions:

The above copyright notice and this permission notice shall be included in
all copies or substantial portions of the Software.

THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF ANY KIND, EXPRESS OR
IMPLIED, INCLUDING BUT NOT LIMITED TO THE WARRANTIES OF MERCHANTABILITY,
FITNESS FOR A PARTICULAR PURPOSE AND NONINFRINGEMENT. IN NO EVENT SHALL THE
AUTHORS OR COPYRIGHT HOLDERS BE LIABLE FOR ANY CLAIM, DAMAGES OR OTHER
LIABILITY, WHETHER IN AN ACTION OF CONTRACT, TORT OR OTHERWISE, ARISING FROM,
OUT OF OR IN CONNECTION WITH THE SOFTWARE OR THE USE OR OTHER DEALINGS IN
THE SOFTWARE.
```

### screenshot_clipboard icon

Original asset: [`common/src/main/resources/icon.png`](https://github.com/comp500/ScreenshotToClipboard/blob/8ea71318ef1ab4c225e63d359d22f538fa059b51/common/src/main/resources/icon.png), resampled to 64 × 64. License: MIT.

```text
MIT License

Copyright (c) 2018 comp500

Permission is hereby granted, free of charge, to any person obtaining a copy
of this software and associated documentation files (the "Software"), to deal
in the Software without restriction, including without limitation the rights
to use, copy, modify, merge, publish, distribute, sublicense, and/or sell
copies of the Software, and to permit persons to whom the Software is
furnished to do so, subject to the following conditions:

The above copyright notice and this permission notice shall be included in all
copies or substantial portions of the Software.

THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF ANY KIND, EXPRESS OR
IMPLIED, INCLUDING BUT NOT LIMITED TO THE WARRANTIES OF MERCHANTABILITY,
FITNESS FOR A PARTICULAR PURPOSE AND NONINFRINGEMENT. IN NO EVENT SHALL THE
AUTHORS OR COPYRIGHT HOLDERS BE LIABLE FOR ANY CLAIM, DAMAGES OR OTHER
LIABILITY, WHETHER IN AN ACTION OF CONTRACT, TORT OR OTHERWISE, ARISING FROM,
OUT OF OR IN CONNECTION WITH THE SOFTWARE OR THE USE OR OTHER DEALINGS IN THE
SOFTWARE.
```

### chat_heads icon

Original asset: [`fabric/src/main/resources/assets/chat_heads/icon.png`](https://github.com/dzwdz/chat_heads/blob/3a42bd9184045e30520ea4069fbda9f4321a0859/fabric/src/main/resources/assets/chat_heads/icon.png). The PNG is resampled to 64 × 64; the unmodified original and modified version are available in `crates/horizon-client-render/assets/mod_icons/`. License: Mozilla Public License 2.0.

```text
Copyright (c) 2020 dzwdz

This Source Code Form is subject to the terms of the Mozilla Public
License, v. 2.0. If a copy of the MPL was not distributed with this
file, You can obtain one at http://mozilla.org/MPL/2.0/.

Mozilla Public License Version 2.0
==================================

1. Definitions
--------------

1.1. "Contributor"
    means each individual or legal entity that creates, contributes to
    the creation of, or owns Covered Software.

1.2. "Contributor Version"
    means the combination of the Contributions of others (if any) used
    by a Contributor and that particular Contributor's Contribution.

1.3. "Contribution"
    means Covered Software of a particular Contributor.

1.4. "Covered Software"
    means Source Code Form to which the initial Contributor has attached
    the notice in Exhibit A, the Executable Form of such Source Code
    Form, and Modifications of such Source Code Form, in each case
    including portions thereof.

1.5. "Incompatible With Secondary Licenses"
    means

    (a) that the initial Contributor has attached the notice described
        in Exhibit B to the Covered Software; or

    (b) that the Covered Software was made available under the terms of
        version 1.1 or earlier of the License, but not also under the
        terms of a Secondary License.

1.6. "Executable Form"
    means any form of the work other than Source Code Form.

1.7. "Larger Work"
    means a work that combines Covered Software with other material, in
    a separate file or files, that is not Covered Software.

1.8. "License"
    means this document.

1.9. "Licensable"
    means having the right to grant, to the maximum extent possible,
    whether at the time of the initial grant or subsequently, any and
    all of the rights conveyed by this License.

1.10. "Modifications"
    means any of the following:

    (a) any file in Source Code Form that results from an addition to,
        deletion from, or modification of the contents of Covered
        Software; or

    (b) any new file in Source Code Form that contains any Covered
        Software.

1.11. "Patent Claims" of a Contributor
    means any patent claim(s), including without limitation, method,
    process, and apparatus claims, in any patent Licensable by such
    Contributor that would be infringed, but for the grant of the
    License, by the making, using, selling, offering for sale, having
    made, import, or transfer of either its Contributions or its
    Contributor Version.

1.12. "Secondary License"
    means either the GNU General Public License, Version 2.0, the GNU
    Lesser General Public License, Version 2.1, the GNU Affero General
    Public License, Version 3.0, or any later versions of those
    licenses.

1.13. "Source Code Form"
    means the form of the work preferred for making modifications.

1.14. "You" (or "Your")
    means an individual or a legal entity exercising rights under this
    License. For legal entities, "You" includes any entity that
    controls, is controlled by, or is under common control with You. For
    purposes of this definition, "control" means (a) the power, direct
    or indirect, to cause the direction or management of such entity,
    whether by contract or otherwise, or (b) ownership of more than
    fifty percent (50%) of the outstanding shares or beneficial
    ownership of such entity.

2. License Grants and Conditions
--------------------------------

2.1. Grants

Each Contributor hereby grants You a world-wide, royalty-free,
non-exclusive license:

(a) under intellectual property rights (other than patent or trademark)
    Licensable by such Contributor to use, reproduce, make available,
    modify, display, perform, distribute, and otherwise exploit its
    Contributions, either on an unmodified basis, with Modifications, or
    as part of a Larger Work; and

(b) under Patent Claims of such Contributor to make, use, sell, offer
    for sale, have made, import, and otherwise transfer either its
    Contributions or its Contributor Version.

2.2. Effective Date

The licenses granted in Section 2.1 with respect to any Contribution
become effective for each Contribution on the date the Contributor first
distributes such Contribution.

2.3. Limitations on Grant Scope

The licenses granted in this Section 2 are the only rights granted under
this License. No additional rights or licenses will be implied from the
distribution or licensing of Covered Software under this License.
Notwithstanding Section 2.1(b) above, no patent license is granted by a
Contributor:

(a) for any code that a Contributor has removed from Covered Software;
    or

(b) for infringements caused by: (i) Your and any other third party's
    modifications of Covered Software, or (ii) the combination of its
    Contributions with other software (except as part of its Contributor
    Version); or

(c) under Patent Claims infringed by Covered Software in the absence of
    its Contributions.

This License does not grant any rights in the trademarks, service marks,
or logos of any Contributor (except as may be necessary to comply with
the notice requirements in Section 3.4).

2.4. Subsequent Licenses

No Contributor makes additional grants as a result of Your choice to
distribute the Covered Software under a subsequent version of this
License (see Section 10.2) or under the terms of a Secondary License (if
permitted under the terms of Section 3.3).

2.5. Representation

Each Contributor represents that the Contributor believes its
Contributions are its original creation(s) or it has sufficient rights
to grant the rights to its Contributions conveyed by this License.

2.6. Fair Use

This License is not intended to limit any rights You have under
applicable copyright doctrines of fair use, fair dealing, or other
equivalents.

2.7. Conditions

Sections 3.1, 3.2, 3.3, and 3.4 are conditions of the licenses granted
in Section 2.1.

3. Responsibilities
-------------------

3.1. Distribution of Source Form

All distribution of Covered Software in Source Code Form, including any
Modifications that You create or to which You contribute, must be under
the terms of this License. You must inform recipients that the Source
Code Form of the Covered Software is governed by the terms of this
License, and how they can obtain a copy of this License. You may not
attempt to alter or restrict the recipients' rights in the Source Code
Form.

3.2. Distribution of Executable Form

If You distribute Covered Software in Executable Form then:

(a) such Covered Software must also be made available in Source Code
    Form, as described in Section 3.1, and You must inform recipients of
    the Executable Form how they can obtain a copy of such Source Code
    Form by reasonable means in a timely manner, at a charge no more
    than the cost of distribution to the recipient; and

(b) You may distribute such Executable Form under the terms of this
    License, or sublicense it under different terms, provided that the
    license for the Executable Form does not attempt to limit or alter
    the recipients' rights in the Source Code Form under this License.

3.3. Distribution of a Larger Work

You may create and distribute a Larger Work under terms of Your choice,
provided that You also comply with the requirements of this License for
the Covered Software. If the Larger Work is a combination of Covered
Software with a work governed by one or more Secondary Licenses, and the
Covered Software is not Incompatible With Secondary Licenses, this
License permits You to additionally distribute such Covered Software
under the terms of such Secondary License(s), so that the recipient of
the Larger Work may, at their option, further distribute the Covered
Software under the terms of either this License or such Secondary
License(s).

3.4. Notices

You may not remove or alter the substance of any license notices
(including copyright notices, patent notices, disclaimers of warranty,
or limitations of liability) contained within the Source Code Form of
the Covered Software, except that You may alter any license notices to
the extent required to remedy known factual inaccuracies.

3.5. Application of Additional Terms

You may choose to offer, and to charge a fee for, warranty, support,
indemnity or liability obligations to one or more recipients of Covered
Software. However, You may do so only on Your own behalf, and not on
behalf of any Contributor. You must make it absolutely clear that any
such warranty, support, indemnity, or liability obligation is offered by
You alone, and You hereby agree to indemnify every Contributor for any
liability incurred by such Contributor as a result of warranty, support,
indemnity or liability terms You offer. You may include additional
disclaimers of warranty and limitations of liability specific to any
jurisdiction.

4. Inability to Comply Due to Statute or Regulation
---------------------------------------------------

If it is impossible for You to comply with any of the terms of this
License with respect to some or all of the Covered Software due to
statute, judicial order, or regulation then You must: (a) comply with
the terms of this License to the maximum extent possible; and (b)
describe the limitations and the code they affect. Such description must
be placed in a text file included with all distributions of the Covered
Software under this License. Except to the extent prohibited by statute
or regulation, such description must be sufficiently detailed for a
recipient of ordinary skill to be able to understand it.

5. Termination
--------------

5.1. The rights granted under this License will terminate automatically
if You fail to comply with any of its terms. However, if You become
compliant, then the rights granted under this License from a particular
Contributor are reinstated (a) provisionally, unless and until such
Contributor explicitly and finally terminates Your grants, and (b) on an
ongoing basis, if such Contributor fails to notify You of the
non-compliance by some reasonable means prior to 60 days after You have
come back into compliance. Moreover, Your grants from a particular
Contributor are reinstated on an ongoing basis if such Contributor
notifies You of the non-compliance by some reasonable means, this is the
first time You have received notice of non-compliance with this License
from such Contributor, and You become compliant prior to 30 days after
Your receipt of the notice.

5.2. If You initiate litigation against any entity by asserting a patent
infringement claim (excluding declaratory judgment actions,
counter-claims, and cross-claims) alleging that a Contributor Version
directly or indirectly infringes any patent, then the rights granted to
You by any and all Contributors for the Covered Software under Section
2.1 of this License shall terminate.

5.3. In the event of termination under Sections 5.1 or 5.2 above, all
end user license agreements (excluding distributors and resellers) which
have been validly granted by You or Your distributors under this License
prior to termination shall survive termination.

************************************************************************
*                                                                      *
*  6. Disclaimer of Warranty                                           *
*  -------------------------                                           *
*                                                                      *
*  Covered Software is provided under this License on an "as is"       *
*  basis, without warranty of any kind, either expressed, implied, or  *
*  statutory, including, without limitation, warranties that the       *
*  Covered Software is free of defects, merchantable, fit for a        *
*  particular purpose or non-infringing. The entire risk as to the     *
*  quality and performance of the Covered Software is with You.        *
*  Should any Covered Software prove defective in any respect, You     *
*  (not any Contributor) assume the cost of any necessary servicing,   *
*  repair, or correction. This disclaimer of warranty constitutes an   *
*  essential part of this License. No use of any Covered Software is   *
*  authorized under this License except under this disclaimer.         *
*                                                                      *
************************************************************************

************************************************************************
*                                                                      *
*  7. Limitation of Liability                                          *
*  --------------------------                                          *
*                                                                      *
*  Under no circumstances and under no legal theory, whether tort      *
*  (including negligence), contract, or otherwise, shall any           *
*  Contributor, or anyone who distributes Covered Software as          *
*  permitted above, be liable to You for any direct, indirect,         *
*  special, incidental, or consequential damages of any character      *
*  including, without limitation, damages for lost profits, loss of    *
*  goodwill, work stoppage, computer failure or malfunction, or any    *
*  and all other commercial damages or losses, even if such party      *
*  shall have been informed of the possibility of such damages. This   *
*  limitation of liability shall not apply to liability for death or   *
*  personal injury resulting from such party's negligence to the       *
*  extent applicable law prohibits such limitation. Some               *
*  jurisdictions do not allow the exclusion or limitation of           *
*  incidental or consequential damages, so this exclusion and          *
*  limitation may not apply to You.                                    *
*                                                                      *
************************************************************************

8. Litigation
-------------

Any litigation relating to this License may be brought only in the
courts of a jurisdiction where the defendant maintains its principal
place of business and such litigation shall be governed by laws of that
jurisdiction, without reference to its conflict-of-law provisions.
Nothing in this Section shall prevent a party's ability to bring
cross-claims or counter-claims.

9. Miscellaneous
----------------

This License represents the complete agreement concerning the subject
matter hereof. If any provision of this License is held to be
unenforceable, such provision shall be reformed only to the extent
necessary to make it enforceable. Any law or regulation which provides
that the language of a contract shall be construed against the drafter
shall not be used to construe this License against a Contributor.

10. Versions of the License
---------------------------

10.1. New Versions

Mozilla Foundation is the license steward. Except as provided in Section
10.3, no one other than the license steward has the right to modify or
publish new versions of this License. Each version will be given a
distinguishing version number.

10.2. Effect of New Versions

You may distribute the Covered Software under the terms of the version
of the License under which You originally received the Covered Software,
or under the terms of any subsequent version published by the license
steward.

10.3. Modified Versions

If you create software not governed by this License, and you want to
create a new license for such software, you may create and use a
modified version of this License if you rename the license and remove
any references to the name of the license steward (except to note that
such modified license differs from this License).

10.4. Distributing Source Code Form that is Incompatible With Secondary
Licenses

If You choose to distribute Source Code Form that is Incompatible With
Secondary Licenses under the terms of this version of the License, the
notice described in Exhibit B of this License must be attached.

Exhibit A - Source Code Form License Notice
-------------------------------------------

  This Source Code Form is subject to the terms of the Mozilla Public
  License, v. 2.0. If a copy of the MPL was not distributed with this
  file, You can obtain one at http://mozilla.org/MPL/2.0/.

If it is not possible or desirable to put the notice in a particular
file, then You may include the notice in a location (such as a LICENSE
file in a relevant directory) where a recipient would be likely to look
for such a notice.

You may add additional accurate notices of copyright ownership.

Exhibit B - "Incompatible With Secondary Licenses" Notice
---------------------------------------------------------

  This Source Code Form is "Incompatible With Secondary Licenses", as
  defined by the Mozilla Public License, v. 2.0.
```

### better_clouds icon

Original asset: [`common/src/main/resources/assets/betterclouds/icon.png`](https://github.com/Qendolin/better-clouds/blob/21fce7c3ef86ed7e7275da5b36f8d97ce9c60299/common/src/main/resources/assets/betterclouds/icon.png). The PNG is resampled to 64 × 64; the unmodified original and modified version are available in `crates/horizon-client-render/assets/mod_icons/`. License: Mozilla Public License 2.0.

```text
Mozilla Public License Version 2.0
==================================

1. Definitions
--------------

1.1. "Contributor"
    means each individual or legal entity that creates, contributes to
    the creation of, or owns Covered Software.

1.2. "Contributor Version"
    means the combination of the Contributions of others (if any) used
    by a Contributor and that particular Contributor's Contribution.

1.3. "Contribution"
    means Covered Software of a particular Contributor.

1.4. "Covered Software"
    means Source Code Form to which the initial Contributor has attached
    the notice in Exhibit A, the Executable Form of such Source Code
    Form, and Modifications of such Source Code Form, in each case
    including portions thereof.

1.5. "Incompatible With Secondary Licenses"
    means

    (a) that the initial Contributor has attached the notice described
        in Exhibit B to the Covered Software; or

    (b) that the Covered Software was made available under the terms of
        version 1.1 or earlier of the License, but not also under the
        terms of a Secondary License.

1.6. "Executable Form"
    means any form of the work other than Source Code Form.

1.7. "Larger Work"
    means a work that combines Covered Software with other material, in
    a separate file or files, that is not Covered Software.

1.8. "License"
    means this document.

1.9. "Licensable"
    means having the right to grant, to the maximum extent possible,
    whether at the time of the initial grant or subsequently, any and
    all of the rights conveyed by this License.

1.10. "Modifications"
    means any of the following:

    (a) any file in Source Code Form that results from an addition to,
        deletion from, or modification of the contents of Covered
        Software; or

    (b) any new file in Source Code Form that contains any Covered
        Software.

1.11. "Patent Claims" of a Contributor
    means any patent claim(s), including without limitation, method,
    process, and apparatus claims, in any patent Licensable by such
    Contributor that would be infringed, but for the grant of the
    License, by the making, using, selling, offering for sale, having
    made, import, or transfer of either its Contributions or its
    Contributor Version.

1.12. "Secondary License"
    means either the GNU General Public License, Version 2.0, the GNU
    Lesser General Public License, Version 2.1, the GNU Affero General
    Public License, Version 3.0, or any later versions of those
    licenses.

1.13. "Source Code Form"
    means the form of the work preferred for making modifications.

1.14. "You" (or "Your")
    means an individual or a legal entity exercising rights under this
    License. For legal entities, "You" includes any entity that
    controls, is controlled by, or is under common control with You. For
    purposes of this definition, "control" means (a) the power, direct
    or indirect, to cause the direction or management of such entity,
    whether by contract or otherwise, or (b) ownership of more than
    fifty percent (50%) of the outstanding shares or beneficial
    ownership of such entity.

2. License Grants and Conditions
--------------------------------

2.1. Grants

Each Contributor hereby grants You a world-wide, royalty-free,
non-exclusive license:

(a) under intellectual property rights (other than patent or trademark)
    Licensable by such Contributor to use, reproduce, make available,
    modify, display, perform, distribute, and otherwise exploit its
    Contributions, either on an unmodified basis, with Modifications, or
    as part of a Larger Work; and

(b) under Patent Claims of such Contributor to make, use, sell, offer
    for sale, have made, import, and otherwise transfer either its
    Contributions or its Contributor Version.

2.2. Effective Date

The licenses granted in Section 2.1 with respect to any Contribution
become effective for each Contribution on the date the Contributor first
distributes such Contribution.

2.3. Limitations on Grant Scope

The licenses granted in this Section 2 are the only rights granted under
this License. No additional rights or licenses will be implied from the
distribution or licensing of Covered Software under this License.
Notwithstanding Section 2.1(b) above, no patent license is granted by a
Contributor:

(a) for any code that a Contributor has removed from Covered Software;
    or

(b) for infringements caused by: (i) Your and any other third party's
    modifications of Covered Software, or (ii) the combination of its
    Contributions with other software (except as part of its Contributor
    Version); or

(c) under Patent Claims infringed by Covered Software in the absence of
    its Contributions.

This License does not grant any rights in the trademarks, service marks,
or logos of any Contributor (except as may be necessary to comply with
the notice requirements in Section 3.4).

2.4. Subsequent Licenses

No Contributor makes additional grants as a result of Your choice to
distribute the Covered Software under a subsequent version of this
License (see Section 10.2) or under the terms of a Secondary License (if
permitted under the terms of Section 3.3).

2.5. Representation

Each Contributor represents that the Contributor believes its
Contributions are its original creation(s) or it has sufficient rights
to grant the rights to its Contributions conveyed by this License.

2.6. Fair Use

This License is not intended to limit any rights You have under
applicable copyright doctrines of fair use, fair dealing, or other
equivalents.

2.7. Conditions

Sections 3.1, 3.2, 3.3, and 3.4 are conditions of the licenses granted
in Section 2.1.

3. Responsibilities
-------------------

3.1. Distribution of Source Form

All distribution of Covered Software in Source Code Form, including any
Modifications that You create or to which You contribute, must be under
the terms of this License. You must inform recipients that the Source
Code Form of the Covered Software is governed by the terms of this
License, and how they can obtain a copy of this License. You may not
attempt to alter or restrict the recipients' rights in the Source Code
Form.

3.2. Distribution of Executable Form

If You distribute Covered Software in Executable Form then:

(a) such Covered Software must also be made available in Source Code
    Form, as described in Section 3.1, and You must inform recipients of
    the Executable Form how they can obtain a copy of such Source Code
    Form by reasonable means in a timely manner, at a charge no more
    than the cost of distribution to the recipient; and

(b) You may distribute such Executable Form under the terms of this
    License, or sublicense it under different terms, provided that the
    license for the Executable Form does not attempt to limit or alter
    the recipients' rights in the Source Code Form under this License.

3.3. Distribution of a Larger Work

You may create and distribute a Larger Work under terms of Your choice,
provided that You also comply with the requirements of this License for
the Covered Software. If the Larger Work is a combination of Covered
Software with a work governed by one or more Secondary Licenses, and the
Covered Software is not Incompatible With Secondary Licenses, this
License permits You to additionally distribute such Covered Software
under the terms of such Secondary License(s), so that the recipient of
the Larger Work may, at their option, further distribute the Covered
Software under the terms of either this License or such Secondary
License(s).

3.4. Notices

You may not remove or alter the substance of any license notices
(including copyright notices, patent notices, disclaimers of warranty,
or limitations of liability) contained within the Source Code Form of
the Covered Software, except that You may alter any license notices to
the extent required to remedy known factual inaccuracies.

3.5. Application of Additional Terms

You may choose to offer, and to charge a fee for, warranty, support,
indemnity or liability obligations to one or more recipients of Covered
Software. However, You may do so only on Your own behalf, and not on
behalf of any Contributor. You must make it absolutely clear that any
such warranty, support, indemnity, or liability obligation is offered by
You alone, and You hereby agree to indemnify every Contributor for any
liability incurred by such Contributor as a result of warranty, support,
indemnity or liability terms You offer. You may include additional
disclaimers of warranty and limitations of liability specific to any
jurisdiction.

4. Inability to Comply Due to Statute or Regulation
---------------------------------------------------

If it is impossible for You to comply with any of the terms of this
License with respect to some or all of the Covered Software due to
statute, judicial order, or regulation then You must: (a) comply with
the terms of this License to the maximum extent possible; and (b)
describe the limitations and the code they affect. Such description must
be placed in a text file included with all distributions of the Covered
Software under this License. Except to the extent prohibited by statute
or regulation, such description must be sufficiently detailed for a
recipient of ordinary skill to be able to understand it.

5. Termination
--------------

5.1. The rights granted under this License will terminate automatically
if You fail to comply with any of its terms. However, if You become
compliant, then the rights granted under this License from a particular
Contributor are reinstated (a) provisionally, unless and until such
Contributor explicitly and finally terminates Your grants, and (b) on an
ongoing basis, if such Contributor fails to notify You of the
non-compliance by some reasonable means prior to 60 days after You have
come back into compliance. Moreover, Your grants from a particular
Contributor are reinstated on an ongoing basis if such Contributor
notifies You of the non-compliance by some reasonable means, this is the
first time You have received notice of non-compliance with this License
from such Contributor, and You become compliant prior to 30 days after
Your receipt of the notice.

5.2. If You initiate litigation against any entity by asserting a patent
infringement claim (excluding declaratory judgment actions,
counter-claims, and cross-claims) alleging that a Contributor Version
directly or indirectly infringes any patent, then the rights granted to
You by any and all Contributors for the Covered Software under Section
2.1 of this License shall terminate.

5.3. In the event of termination under Sections 5.1 or 5.2 above, all
end user license agreements (excluding distributors and resellers) which
have been validly granted by You or Your distributors under this License
prior to termination shall survive termination.

************************************************************************
*                                                                      *
*  6. Disclaimer of Warranty                                           *
*  -------------------------                                           *
*                                                                      *
*  Covered Software is provided under this License on an "as is"       *
*  basis, without warranty of any kind, either expressed, implied, or  *
*  statutory, including, without limitation, warranties that the       *
*  Covered Software is free of defects, merchantable, fit for a        *
*  particular purpose or non-infringing. The entire risk as to the     *
*  quality and performance of the Covered Software is with You.        *
*  Should any Covered Software prove defective in any respect, You     *
*  (not any Contributor) assume the cost of any necessary servicing,   *
*  repair, or correction. This disclaimer of warranty constitutes an   *
*  essential part of this License. No use of any Covered Software is   *
*  authorized under this License except under this disclaimer.         *
*                                                                      *
************************************************************************

************************************************************************
*                                                                      *
*  7. Limitation of Liability                                          *
*  --------------------------                                          *
*                                                                      *
*  Under no circumstances and under no legal theory, whether tort      *
*  (including negligence), contract, or otherwise, shall any           *
*  Contributor, or anyone who distributes Covered Software as          *
*  permitted above, be liable to You for any direct, indirect,         *
*  special, incidental, or consequential damages of any character      *
*  including, without limitation, damages for lost profits, loss of    *
*  goodwill, work stoppage, computer failure or malfunction, or any    *
*  and all other commercial damages or losses, even if such party      *
*  shall have been informed of the possibility of such damages. This   *
*  limitation of liability shall not apply to liability for death or   *
*  personal injury resulting from such party's negligence to the       *
*  extent applicable law prohibits such limitation. Some               *
*  jurisdictions do not allow the exclusion or limitation of           *
*  incidental or consequential damages, so this exclusion and          *
*  limitation may not apply to You.                                    *
*                                                                      *
************************************************************************

8. Litigation
-------------

Any litigation relating to this License may be brought only in the
courts of a jurisdiction where the defendant maintains its principal
place of business and such litigation shall be governed by laws of that
jurisdiction, without reference to its conflict-of-law provisions.
Nothing in this Section shall prevent a party's ability to bring
cross-claims or counter-claims.

9. Miscellaneous
----------------

This License represents the complete agreement concerning the subject
matter hereof. If any provision of this License is held to be
unenforceable, such provision shall be reformed only to the extent
necessary to make it enforceable. Any law or regulation which provides
that the language of a contract shall be construed against the drafter
shall not be used to construe this License against a Contributor.

10. Versions of the License
---------------------------

10.1. New Versions

Mozilla Foundation is the license steward. Except as provided in Section
10.3, no one other than the license steward has the right to modify or
publish new versions of this License. Each version will be given a
distinguishing version number.

10.2. Effect of New Versions

You may distribute the Covered Software under the terms of the version
of the License under which You originally received the Covered Software,
or under the terms of any subsequent version published by the license
steward.

10.3. Modified Versions

If you create software not governed by this License, and you want to
create a new license for such software, you may create and use a
modified version of this License if you rename the license and remove
any references to the name of the license steward (except to note that
such modified license differs from this License).

10.4. Distributing Source Code Form that is Incompatible With Secondary
Licenses

If You choose to distribute Source Code Form that is Incompatible With
Secondary Licenses under the terms of this version of the License, the
notice described in Exhibit B of this License must be attached.

Exhibit A - Source Code Form License Notice
-------------------------------------------

  This Source Code Form is subject to the terms of the Mozilla Public
  License, v. 2.0. If a copy of the MPL was not distributed with this
  file, You can obtain one at http://mozilla.org/MPL/2.0/.

If it is not possible or desirable to put the notice in a particular
file, then You may include the notice in a location (such as a LICENSE
file in a relevant directory) where a recipient would be likely to look
for such a notice.

You may add additional accurate notices of copyright ownership.

Exhibit B - "Incompatible With Secondary Licenses" Notice
---------------------------------------------------------

  This Source Code Form is "Incompatible With Secondary Licenses", as
  defined by the Mozilla Public License, v. 2.0.
```

## Horizon Zoom native mod 2.0.0

`crates/horizon-client/src/game/official_zoom.rs` is the runtime of Horizon's
[Zoom 2.0.0](https://github.com/horizon-rust/horizon-mods/tree/22c6f14ea60be2b17a642b44a2aff44a98e3733d/native/zoom),
by Ludovic Marie and the Horizon contributors, under the MIT License. The full
license is in `crates/horizon-client/src/game/licenses/HORIZON-ZOOM-MIT.txt`.
Volt adds a profile-state reset, integrates its hooks, and provides profile
activation on Android; its
Horizon key and settings remain distinct from Volt's Zoomify implementation.

```text
MIT License

Copyright (c) 2026 Ludovic Marie and the Horizon contributors

Permission is hereby granted, free of charge, to any person obtaining a copy
of this software and associated documentation files (the "Software"), to deal
in the Software without restriction, including without limitation the rights
to use, copy, modify, merge, publish, distribute, sublicense, and/or sell
copies of the Software, and to permit persons to whom the Software is
furnished to do so, subject to the following conditions:

The above copyright notice and this permission notice shall be included in all
copies or substantial portions of the Software.

THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF ANY KIND, EXPRESS OR
IMPLIED, INCLUDING BUT NOT LIMITED TO THE WARRANTIES OF MERCHANTABILITY,
FITNESS FOR A PARTICULAR PURPOSE AND NONINFRINGEMENT. IN NO EVENT SHALL THE
AUTHORS OR COPYRIGHT HOLDERS BE LIABLE FOR ANY CLAIM, DAMAGES OR OTHER
LIABILITY, WHETHER IN AN ACTION OF CONTRACT, TORT OR OTHERWISE, ARISING FROM,
OUT OF OR IN CONNECTION WITH THE SOFTWARE OR THE USE OR OTHER DEALINGS IN THE
SOFTWARE.
```

## Horizon FPS Counter, Keystrokes and Minimap native mods

`crates/horizon-client/src/game/official_{fps_overlay,keystrokes,minimap}.rs`
contain the official Horizon 2.0.0 runtimes from
[commit 22c6f14ea60be2b17a642b44a2aff44a98e3733d](https://github.com/horizon-rust/horizon-mods/tree/22c6f14ea60be2b17a642b44a2aff44a98e3733d/native),
by Ludovic Marie and the Horizon contributors. The same MIT license printed
above applies. Volt adds a per-profile state reset and Android integration.

## JEI for Horizon native mod 0.1.0

Release APKs weave the published registry bundle
[`jei-0.1.0.hnmod`](https://github.com/horizon-rust/horizon-mods/releases/tag/jei-0.1.0)
(SHA-256 `c5590e0d4618d435fc0c56abcc9cdabf253be694710068a3dd4643ac25466b5b`)
into the engine library at build time. JEI for Horizon is by enderliker
([source](https://github.com/enderliker/horizon-jei)), under the MIT License
(the same license text printed above, Copyright (c) 2026 enderliker). The
weave happens on a no-secrets CI runner and the player can turn the mod off
per profile in-game under Mods.

## Horizon Physics native mod 0.15.1

Release APKs weave the published registry bundle
[`horizon_physics-0.15.1.hnmod`](https://github.com/horizon-rust/horizon-mods/releases/tag/horizon_physics-0.15.1)
(SHA-256 `d7cf1201f0a47a852aa1a4c9684ef9e658ec3c86852104d44716d05f11281ca3`)
into the engine library at build time. Horizon Physics is by tariksabanuz
([source](https://github.com/tariksabanuz/PhysicsModHorizon)), under the MIT
License (the same license text printed above, Copyright (c) 2026
tariksabanuz). It is a clean-room work inspired by PhysicsMod and contains
none of its code or assets. The weave happens on a no-secrets CI runner and
the player can turn the mod off per profile in-game under Mods.

## Jade native mod 26.3.5

Release APKs weave the published registry bundle
[`jade-26.3.5.hnmod`](https://github.com/horizon-rust/horizon-mods/releases/tag/jade-26.3.5)
(SHA-256 `676a129f55db075284313a39ff4387674c426ebd9be04a8830ffb8955281c7ec`)
into the engine library at build time. Jade for Horizon is by Joeypc6
([source](https://github.com/Joeypc6/Jade-Horizon)), an unofficial Rust port
of [Jade](https://github.com/Snownee/Jade) by Snownee; no original Jade code
runs, and the port is neither made nor endorsed by Snownee. Both the port and
Jade are licensed under the Creative Commons
Attribution-NonCommercial-ShareAlike 4.0 International License
(CC BY-NC-SA 4.0, <https://creativecommons.org/licenses/by-nc-sa/4.0/>):
attribution to Snownee and Joeypc6 as above, noncommercial use only, and
adaptations of the mod must carry the same license. Volt Launcher is
distributed free of charge and noncommercially. The weave happens on a
no-secrets CI runner and the player can turn the mod off per profile in-game
under Mods.

## Veinminer native mod 0.3.2

Release APKs weave the published registry bundle
[`veinminer-0.3.2.hnmod`](https://github.com/horizon-rust/horizon-mods/releases/tag/veinminer-0.3.2)
(SHA-256 `31a4249da1b35972ae8b34201373c86352f4adc2f5cb3e015749d1fbfecf00a0`)
into the engine library at build time. Veinminer for Horizon is by
levzzz5154 ([source](https://github.com/levzzz5154/horizon-veinminer)),
under the MIT License (the same license text printed above, Copyright (c)
2026 levzzz5154). It is independently written Rust code inspired by
[MiraculixxT's Veinminer](https://github.com/MiraculixxT/Veinminer) and
bundles no upstream mod code or artwork. The weave happens on a no-secrets
CI runner and the player can turn the mod off per profile in-game under
Mods.

## Additional woven registry native mods (MIT)

Release APKs also weave these published registry bundles into the engine
library at build time, each under the MIT License (the same license text
printed above, copyright their listed authors). The weave happens on a
no-secrets CI runner and each mod can be turned off per profile in-game
under Mods.

- **appleskin 1.0.0** by soufi — [appleskin-1.0.0.hnmod](https://github.com/horizon-rust/horizon-mods/releases/tag/appleskin-1.0.0), SHA-256 `3c25b916f74f47faf40070a8654c546d15cb79ac3947f140e257f4cb041aeb96`, [source](https://github.com/soufied/AppleSkin-Horizon)
- **status_effect_hud 1.0.0** by CayMac — [status_effect_hud-1.0.0.hnmod](https://github.com/horizon-rust/horizon-mods/releases/tag/status_effect_hud-1.0.0), SHA-256 `afa30125bb8b402892f1719e550b66d4eb4d7264895f71bb9c98a4caf5c7b432`, [source](https://github.com/CayMac97/status-effect-hud)
- **toggle_sprint 1.0.0** by CayMac — [toggle_sprint-1.0.0.hnmod](https://github.com/horizon-rust/horizon-mods/releases/tag/toggle_sprint-1.0.0), SHA-256 `b2a4190f9c19adbceae4a639342b7630bfc3047339eabf57673b7807db5edc6a`, [source](https://github.com/CayMac97/toggle-sprint)
- **inventory_profiles 1.0.0** by CayMac — [inventory_profiles-1.0.0.hnmod](https://github.com/horizon-rust/horizon-mods/releases/tag/inventory_profiles-1.0.0), SHA-256 `1e5d1c75d05230172820b5bc016d7d3ac792adf473de7dbfb84394d8b795a633`, [source](https://github.com/CayMac97/inventory-profiles)
- **shulker_peek 0.1.0** by vstxx — [shulker_peek-0.1.0.hnmod](https://github.com/horizon-rust/horizon-mods/releases/tag/shulker_peek-0.1.0), SHA-256 `e4e33cb698d91042b2ed2f93a1c935f279d16d83988115d53c0f33530cabeab5`, [source](https://github.com/vstxx/shulkerpeek)
- **mousetricks 0.2.0** by d3grando — [mousetricks-0.2.0.hnmod](https://github.com/horizon-rust/horizon-mods/releases/tag/mousetricks-0.2.0), SHA-256 `48c60955371963f8e320a9efbe8285bb276d255c29178a8a021e7e2b47535cbe`, [source](https://github.com/d3grando/mousetricks)
- **itemnew 0.1.1** by d3grando — [itemnew-0.1.1.hnmod](https://github.com/horizon-rust/horizon-mods/releases/tag/itemnew-0.1.1), SHA-256 `cdc68c9911736703ebc4b8b81895a719edd74ba9d08fe01ddb34fe09b8d490e9`, [source](https://github.com/d3grando/itemnew)
- **eatinganimation 0.1.0** by soufi — [eatinganimation-0.1.0.hnmod](https://github.com/horizon-rust/horizon-mods/releases/tag/eatinganimation-0.1.0), SHA-256 `07ef4fde3ebbc4f132aab593b12187bce15e1a46689ab40bd3067c685aca85b9`, [source](https://github.com/soufied/EatingAnimation-Horizon)
- **enchantmenthints 0.1.0** by d3grando — [enchantmenthints-0.1.0.hnmod](https://github.com/horizon-rust/horizon-mods/releases/tag/enchantmenthints-0.1.0), SHA-256 `6f1b80a7c22e02b086d5eea2ed6f7c5481b3e40d6a974bf9e382a3df4a2de9df`, [source](https://github.com/d3grando/enchantmenthints)
- **light_overlay 1.0.0** by CayMac — [light_overlay-1.0.0.hnmod](https://github.com/horizon-rust/horizon-mods/releases/tag/light_overlay-1.0.0), SHA-256 `b4aeb56b13d9c73df267a1dc7b1f588f4db39bfd55f7beaef97a014fffb5b076`, [source](https://github.com/CayMac97/light-overlay)
- **right_click_crops 1.0.0** by AyanoKaito — [right_click_crops-1.0.0.hnmod](https://github.com/horizon-rust/horizon-mods/releases/tag/right_click_crops-1.0.0), SHA-256 `681ce896c50dd041132900e7bbab71167f133b3f3862f86e8b1a0cac52754b97`, [source](https://github.com/AyanoKaito/horizon-right-click-crops)
- **smartf3 0.1.0** by d3grando — [smartf3-0.1.0.hnmod](https://github.com/horizon-rust/horizon-mods/releases/tag/smartf3-0.1.0), SHA-256 `666fc77a12411c8b525dca3d3d478595e760b73c48b62b936fb6026419547929`, [source](https://github.com/d3grando/smartf3)
- **armor_status 1.0.0** by CayMac — [armor_status-1.0.0.hnmod](https://github.com/horizon-rust/horizon-mods/releases/tag/armor_status-1.0.0), SHA-256 `82c8672fa2c43d7946a26a560fd56fdd8f34beecf4bcb459c2d508b3d09eeee6`, [source](https://github.com/CayMac97/armor-status)
- **crosshair_indicator 1.0.0** by CayMac — [crosshair_indicator-1.0.0.hnmod](https://github.com/horizon-rust/horizon-mods/releases/tag/crosshair_indicator-1.0.0), SHA-256 `96beba1f7c94fca492b552fb12af0d20047dc1799f0801607841d56b12786226`, [source](https://github.com/CayMac97/crosshair-indicator)
- **hitbox_plus 1.3.0** by taxtii, itzglecni — [hitbox_plus-1.3.0.hnmod](https://github.com/horizon-rust/horizon-mods/releases/tag/hitbox_plus-1.3.0), SHA-256 `8d1b95e7461e38fc40994b35a82c1f753a58e9751ce27af1dc101a4717dba8a4`, [source](https://github.com/taxtii/hitbox_plus)
- **invisible_frames 0.1.1** by AltMugen — [invisible_frames-0.1.1.hnmod](https://github.com/horizon-rust/horizon-mods/releases/tag/invisible_frames-0.1.1), SHA-256 `76128b48ce5c826007379c85634b934ae6bad51c134fbaead7c46b36fecd9274`, [source](https://github.com/AltMugen/horizon-invisible-frames)
- **freelook 0.3.1** by Krzysix17 — [freelook-0.3.1.hnmod](https://github.com/horizon-rust/horizon-mods/releases/tag/freelook-0.3.1), SHA-256 `9c0c92ce9f0fc8e7bc1bd34349e26d42b32b8abd2c7a09ec710936f9b6ce9aaf`, [source](https://github.com/Krzysix17/horizon-freelook)
- **voicechat 1.0.0** by Nirokx — [voicechat-1.0.0.hnmod](https://github.com/horizon-rust/horizon-mods/releases/tag/voicechat-1.0.0), SHA-256 `7a12ef9e691d10128ea081906a313d597c67f22897380618871477f9dd8eabf1`, [source](https://github.com/Nirokx/horizon-voice-chat)
- **worldedit 1.2.1** by _cykoo_ — [worldedit-1.2.1.hnmod](https://github.com/horizon-rust/horizon-mods/releases/tag/worldedit-1.2.1), SHA-256 `b2bb3019ecd4d5392644c54c70a87d6c3277538a6e18d0c311f3a051f7d2f015`, [source](https://github.com/XnotCykoX/WorldEdit-Horizon)
- **horizonmatica 0.1.1** by keksi2k — [horizonmatica-0.1.1.hnmod](https://github.com/horizon-rust/horizon-mods/releases/tag/horizonmatica-0.1.1), SHA-256 `2ddc562dd2ea0f9a76a47eb761c64c803def68a8c81b92cb5d15ec26c5d432aa`, [source](https://github.com/keksi2k/horizonmatica)
- **visuality_horizon 1.0.2** by BenjiSyS — [visuality_horizon-1.0.2.hnmod](https://github.com/horizon-rust/horizon-mods/releases/tag/visuality_horizon-1.0.2), SHA-256 `b764f15a062e44b27c55f478b2d02e163bfb01c3d8e222fcd6d83bcc66f19a0d`, [source](https://github.com/BenjiSyS/visuality-horizon)
- **held_item_info 0.1.0** by AltMugen — [held_item_info-0.1.0.hnmod](https://github.com/horizon-rust/horizon-mods/releases/tag/held_item_info-0.1.0), SHA-256 `37a11d98f0afabe37f8ffa9982dc710ad2985817db6b5e094d807c24d30a6a6a`, [source](https://github.com/AltMugen/horizon-held-item-info)
- **carryblock 0.1.2** by d3grando — [carryblock-0.1.2.hnmod](https://github.com/horizon-rust/horizon-mods/releases/tag/carryblock-0.1.2), SHA-256 `9a6ebd26f640a477815862eb36846fe3289046d0f4c06ef3a815f463ca9a3701`, [source](https://github.com/d3grando/carryblock)
- **chatheads 0.1.0** by Domanffe — [chatheads-0.1.0.hnmod](https://github.com/horizon-rust/horizon-mods/releases/tag/chatheads-0.1.0), SHA-256 `c20605c24e29a762b9aea85cba1b0cc7955955a40a031efae70a5b01e4e79df4`, [source](https://github.com/Domanffe/chatheads)
- **fast_items 0.1.0** by Lyvt — [fast_items-0.1.0.hnmod](https://github.com/horizon-rust/horizon-mods/releases/tag/fast_items-0.1.0), SHA-256 `c3acd8adfeac6450c3a13b5172c8abb298cb3ceb355dfc539b4bdd653ef15502`, [source](https://github.com/Lyvt-Dev/FastItems-for-Horizon)
- **customhud 1.6.0** by SCARABOCCHIO100 — [customhud-1.6.0.hnmod](https://github.com/horizon-rust/horizon-mods/releases/tag/customhud-1.6.0), SHA-256 `ab255d3c722b40f4d068a762857d6c027c11381308bfe23196e2974307814a7f`, [source](https://github.com/sonoFrangu/CustomHUD-Horizon)
- **coords_hud 2.0.0** by Horizon — [coords_hud-2.0.0.hnmod](https://github.com/horizon-rust/horizon-mods/releases/tag/coords_hud-2.0.0), SHA-256 `432376fad3062b3517e274d3b8c0d0b52fc5d6de2aee20a0b340c8cab5a8cf08`, [source](https://github.com/horizon-rust/horizon-mods/tree/main/native/coords-hud)
- **biomecompass 0.2.2** by d3grando — [biomecompass-0.2.2.hnmod](https://github.com/horizon-rust/horizon-mods/releases/tag/biomecompass-0.2.2), SHA-256 `d9a4aedcf22318774e1ac6bed6f1e00aca13a38c8dc3426d5a7800e4d8c61bf5`, [source](https://github.com/d3grando/biomecompass)

## Horizon launcher typography

The launcher display face `crates/horizon-theme/fonts/BricolageGrotesqueDisplay-SemiBold.ttf`
comes from the Horizon Nightly source at `f319701e8c9ab657ff562250b3ba07b2d654e3bc`.
Bricolage Grotesque is distributed under the SIL Open Font License 1.1; its
license is `crates/horizon-theme/fonts/OFL-BricolageGrotesque.txt`.

`crates/horizon-theme/src/glass.rs` is from the same Horizon Nightly snapshot,
Copyright (c) 2026 Ludovic Marie and the Horizon contributors, under the MIT
license in the repository's `LICENSE` file.
