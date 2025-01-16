Changelog
=========

[0.22.1](https://github.com/ordinals/ord/releases/tag/0.22.1) - 2024-12-23
--------------------------------------------------------------------------

### Added
- Add `/r/utxo/:outpoint` endpoint ([#4148](https://github.com/ordinals/ord/pull/4148) by [raphjaph](https://github.com/raphjaph))
- Add testnet4 ([#4135](https://github.com/ordinals/ord/pull/4135) by [raphjaph](https://github.com/raphjaph))
- Enable rune burning in wallet ([#4117](https://github.com/ordinals/ord/pull/4117) by [raphjaph](https://github.com/raphjaph))
- Enable redb quick-repair mode ([#4084](https://github.com/ordinals/ord/pull/4084) by [partialord](https://github.com/partialord))

### Changed
- Return `None` for assets when asset index does not exist ([#4141](https://github.com/ordinals/ord/pull/4141) by [raphjaph](https://github.com/raphjaph))
- Show inscription preview video controls on click ([#4139](https://github.com/ordinals/ord/pull/4139) by [casey](https://github.com/casey))
- Do not update index for info command ([#4128](https://github.com/ordinals/ord/pull/4128) by [raphjaph](https://github.com/raphjaph))
- Wait for wallet to load ([#4095](https://github.com/ordinals/ord/pull/4095) by [raphjaph](https://github.com/raphjaph))

### Misc
- Show overflow of `<ol>` in /blocks list ([#4142](https://github.com/ordinals/ord/pull/4142) by [casey](https://github.com/casey))
- Create savepoints more robustly ([#2365](https://github.com/ordinals/ord/pull/2365) by [gmart7t2](https://github.com/gmart7t2))
- Fix deploy for testnet3 ([#4137](https://github.com/ordinals/ord/pull/4137) by [raphjaph](https://github.com/raphjaph))
- Improve wallet sync error messages ([#4126](https://github.com/ordinals/ord/pull/4126) by [raphjaph](https://github.com/raphjaph))
- Link inscription burning documentation ([#4131](https://github.com/ordinals/ord/pull/4131) by [raphjaph](https://github.com/raphjaph))

[0.22.0](https://github.com/ordinals/ord/releases/tag/0.22.0) - 2024-12-10
--------------------------------------------------------------------------

### Added
- Sign for inscription and output ([#4027](https://github.com/ordinals/ord/pull/4027) by [raphjaph](https://github.com/raphjaph))
- Document Inscription URIs ([#4098](https://github.com/ordinals/ord/pull/4098) by [casey](https://github.com/casey))
- Show rune unlock height ([#3580](https://github.com/ordinals/ord/pull/3580) by [raphjaph](https://github.com/raphjaph))
- Add field metaprotocol to api::Inscription ([#4047](https://github.com/ordinals/ord/pull/4047) by [kbehouse](https://github.com/kbehouse))
- Show sat owner address when present ([#4016](https://github.com/ordinals/ord/pull/4016) by [lifofifoX](https://github.com/lifofifoX))

### Changed
- Only burn one sat ([#4063](https://github.com/ordinals/ord/pull/4063) by [onchainguy-btc](https://github.com/onchainguy-btc))

### Fixed
- Set `maxburnamount` when burning and require at least Bitcoin Core 25 ([#4106](https://github.com/ordinals/ord/pull/4106) by [casey](https://github.com/casey))
- Add `palindrome` to `Charm::from_str` ([#4104](https://github.com/ordinals/ord/pull/4104) by [mvdnbrk](https://github.com/mvdnbrk))
- Fix sat off-by-one error in output template ([#4075](https://github.com/ordinals/ord/pull/4075) by [casey](https://github.com/casey))

### Misc
- Style help text ([#4118](https://github.com/ordinals/ord/pull/4118) by [casey](https://github.com/casey))
- Placate clippy ([#4116](https://github.com/ordinals/ord/pull/4116) by [raphjaph](https://github.com/raphjaph))
- Test `Charm` `FromStr` implementation for exhaustiveness ([#4107](https://github.com/ordinals/ord/pull/4107) by [casey](https://github.com/casey))
- Improve `ord wallet send` help message and rename output `outgoing` to `asset` ([#4105](https://github.com/ordinals/ord/pull/4105) by [casey](https://github.com/casey))
- Hide teleburn address ([#4093](https://github.com/ordinals/ord/pull/4093) by [raphjaph](https://github.com/raphjaph))
- Add function to calculate rune unlock height ([#4097](https://github.com/ordinals/ord/pull/4097) by [casey](https://github.com/casey))
- Handle errors when retrieving sat address ([#4094](https://github.com/ordinals/ord/pull/4094) by [casey](https://github.com/casey))
- Fixed error message when using sat or satpoint with batch inscribe ([#4054](https://github.com/ordinals/ord/pull/4054) by [pokrovskyy](https://github.com/pokrovskyy))
- Add replicate and swap recipes ([#4083](https://github.com/ordinals/ord/pull/4083) by [casey](https://github.com/casey))
- Make build script public ([#4085](https://github.com/ordinals/ord/pull/4085) by [arronzhang](https://github.com/arronzhang))
- Update `index.hbs` ([#4090](https://github.com/ordinals/ord/pull/4090) by [raphjaph](https://github.com/raphjaph))
- Update Rust version in Dockerfile ([#4078](https://github.com/ordinals/ord/pull/4078) by [Th0rgal](https://github.com/Th0rgal))
- Update minimum rust version to 1.79.0 ([#4074](https://github.com/ordinals/ord/pull/4074) by [raphjaph](https://github.com/raphjaph))
- Update docs Github Action ([#4069](https://github.com/ordinals/ord/pull/4069) by [raphjaph](https://github.com/raphjaph))
- Pin `bitcoin` to 0.32.3 in `ordinals` crate ([#4066](https://github.com/ordinals/ord/pull/4066) by [casey](https://github.com/casey))

[0.21.3](https://github.com/ordinals/ord/releases/tag/0.21.3) - 2024-11-11
--------------------------------------------------------------------------

### Added
- Get output information by address ([#4056](https://github.com/ordinals/ord/pull/4056) by [raphjaph](https://github.com/raphjaph))
- Allow including metadata when burning inscriptions ([#4045](https://github.com/ordinals/ord/pull/4045) by [casey](https://github.com/casey))
- BIP322 sign file ([#4026](https://github.com/ordinals/ord/pull/4026) by [raphjaph](https://github.com/raphjaph))
- Add `ord wallet split` command for splitting utxos ([#4030](https://github.com/ordinals/ord/pull/4030) by [casey](https://github.com/casey))
- Allow fallback for satpoints and addresses ([#4033](https://github.com/ordinals/ord/pull/4033) by [casey](https://github.com/casey))
- Add palindrome charm ([#4064](https://github.com/ordinals/ord/pull/4064) by [casey](https://github.com/casey))
- Allow restoring wallet with custom timestamp ([#4065](https://github.com/ordinals/ord/pull/4065) by [raphjaph](https://github.com/raphjaph))

### Changed
- Do not chunk runestone data pushes ([#4036](https://github.com/ordinals/ord/pull/4036) by [casey](https://github.com/casey))
- Rescan wallet on restore ([#4041](https://github.com/ordinals/ord/pull/4041) by [casey](https://github.com/casey))

### Misc
- Add assert_html function ([#4058](https://github.com/ordinals/ord/pull/4058) by [casey](https://github.com/casey))
- Identify collapsible nodes with class=collapse ([#4055](https://github.com/ordinals/ord/pull/4055) by [casey](https://github.com/casey))
- Collapse long strings in HTML ([#4053](https://github.com/ordinals/ord/pull/4053) by [casey](https://github.com/casey))
- Add simple taproot HD wallet to mockcore ([#4038](https://github.com/ordinals/ord/pull/4038) by [raphjaph](https://github.com/raphjaph))
- Hide image preview and thumbnail scrollbars ([#4042](https://github.com/ordinals/ord/pull/4042) by [casey](https://github.com/casey))
- Un-pin redb dependency and update to 2.2.0 ([#4032](https://github.com/ordinals/ord/pull/4032) by [casey](https://github.com/casey))

[0.21.2](https://github.com/ordinals/ord/releases/tag/0.21.2) - 2024-10-26
--------------------------------------------------------------------------

### Fixed
- Create change output when inputs containing non-outgoing runes are selected ([#4028](https://github.com/ordinals/ord/pull/4028) by [casey](https://github.com/casey))

### Added
- Show total child count ([#4009](https://github.com/ordinals/ord/pull/4009) by [arik-so](https://github.com/arik-so))
- Add `/r/undelegated-content/<INSCRIPTION_ID>` ([#3932](https://github.com/ordinals/ord/pull/3932) by [elocremarc](https://github.com/elocremarc))
- Add BIP322 `wallet sign` ([#3988](https://github.com/ordinals/ord/pull/3988) by [raphjaph](https://github.com/raphjaph))
- Add `wallet addresses` ([#4005](https://github.com/ordinals/ord/pull/4005) by [raphjaph](https://github.com/raphjaph))
- Show if JSON API is enabled on /status ([#4014](https://github.com/ordinals/ord/pull/4014) by [casey](https://github.com/casey))

### Changed
- Only show rune mint progress during mint ([#4013](https://github.com/ordinals/ord/pull/4013) by [casey](https://github.com/casey))
- Change mint progress to `mints / terms.cap` ([#4012](https://github.com/ordinals/ord/pull/4012) by [casey](https://github.com/casey))

### Misc
- Add more info to `wallet outputs` ([#4019](https://github.com/ordinals/ord/pull/4019) by [raphjaph](https://github.com/raphjaph))
- Add authors to Handbook ([#4018](https://github.com/ordinals/ord/pull/4018) by [raphjaph](https://github.com/raphjaph))
- Document POST method for /inscriptions ([#4017](https://github.com/ordinals/ord/pull/4017) by [cryptoni9n](https://github.com/cryptoni9n))
- Update JSON-API & Recursive documentation ([#3984](https://github.com/ordinals/ord/pull/3984) by [cryptoni9n](https://github.com/cryptoni9n))
- Remove pre-alpha warning from ord help ([#4011](https://github.com/ordinals/ord/pull/4011) by [cryptoni9n](https://github.com/cryptoni9n))
- Update Bitcoin Core install script ([#4007](https://github.com/ordinals/ord/pull/4007) by [raphjaph](https://github.com/raphjaph))

[0.21.1](https://github.com/ordinals/ord/releases/tag/0.21.1) - 2024-10-20
--------------------------------------------------------------------------

### Fixed
- Revert redb to 2.1.3 ([#4003](https://github.com/ordinals/ord/pull/4003) by [raphjaph](https://github.com/raphjaph))

### Changed
- Remove /runes/balances API endpoint ([#3980](https://github.com/ordinals/ord/pull/3980) by [lifofifoX](https://github.com/lifofifoX))

### Misc
- Update rust-bitcoin in ord ([#3962](https://github.com/ordinals/ord/pull/3962) by [raphjaph](https://github.com/raphjaph))

[0.21.0](https://github.com/ordinals/ord/releases/tag/0.21.0) - 2024-10-11
--------------------------------------------------------------------------

### Added
- Add `ord verify` ([#3906](https://github.com/ordinals/ord/pull/3906) by [raphjaph](https://github.com/raphjaph))

### Misc
- Remove regtest.ordinals.net just recipes ([#3978](https://github.com/ordinals/ord/pull/3978) by [casey](https://github.com/casey))
- Refactor burn command ([#3976](https://github.com/ordinals/ord/pull/3976) by [casey](https://github.com/casey))

[0.20.1](https://github.com/ordinals/ord/releases/tag/0.20.1) - 2024-10-03
--------------------------------------------------------------------------

### Fixed
- Fix non-existant output lookup ([#3968](https://github.com/ordinals/ord/pull/3968) by [raphjaph](https://github.com/raphjaph))
- Fix output API struct ([#3957](https://github.com/ordinals/ord/pull/3957) by [raphjaph](https://github.com/raphjaph))
- Start indexing at correct block height ([#3956](https://github.com/ordinals/ord/pull/3956) by [partialord](https://github.com/partialord))
- Fix /output page ([#3948](https://github.com/ordinals/ord/pull/3948) by [raphjaph](https://github.com/raphjaph))

### Added
- Add multi parent support to wallet ([#3228](https://github.com/ordinals/ord/pull/3228) by [raphjaph](https://github.com/raphjaph))
- Implement burn for wallet command ([#3437](https://github.com/ordinals/ord/pull/3437) by [onchainguy-btc](https://github.com/onchainguy-btc))
- Add `/satpoint/<SATPOINT>` endpoint ([#3949](https://github.com/ordinals/ord/pull/3949) by [raphjaph](https://github.com/raphjaph))
- Add inscription examples to handbook ([#3769](https://github.com/ordinals/ord/pull/3769) by [cryptoni9n](https://github.com/cryptoni9n))
- Add inscription index to /status ([#3938](https://github.com/ordinals/ord/pull/3938) by [casey](https://github.com/casey))

### Changed
- Keep sat ranges in low-level format ([#3963](https://github.com/ordinals/ord/pull/3963) by [partialord](https://github.com/partialord))
- Remove dependency on `ord-bitcoincore-rpc` crate ([#3959](https://github.com/ordinals/ord/pull/3959) by [raphjaph](https://github.com/raphjaph))
- Don't log RPC connections to bitcoind ([#3952](https://github.com/ordinals/ord/pull/3952) by [raphjaph](https://github.com/raphjaph))
- Skip serializing None in batch::File ([#3943](https://github.com/ordinals/ord/pull/3943) by [raphjaph](https://github.com/raphjaph))
- Allow scrolling in iframe ([#3947](https://github.com/ordinals/ord/pull/3947) by [raphjaph](https://github.com/raphjaph))
- Put AddressInfo into api module ([#3933](https://github.com/ordinals/ord/pull/3933) by [raphjaph](https://github.com/raphjaph))

### Misc
- Rename parents_values -> parent_values ([#3973](https://github.com/ordinals/ord/pull/3973) by [casey](https://github.com/casey))
- Get parents using `as_slice` instead of converting to `Vec` ([#3972](https://github.com/ordinals/ord/pull/3972) by [casey](https://github.com/casey))
- Remove unnecessary symbols in docs/src/guides/testing.md ([#3945](https://github.com/ordinals/ord/pull/3945) by [tiaoxizhan](https://github.com/tiaoxizhan))
- Fix clippy lint ([#3937](https://github.com/ordinals/ord/pull/3937) by [casey](https://github.com/casey))
- Add test to remind us to fix the UtxoEntry redb type name ([#3934](https://github.com/ordinals/ord/pull/3934) by [casey](https://github.com/casey))
- Bump version to 0.20.0-dev ([#3929](https://github.com/ordinals/ord/pull/3929) by [casey](https://github.com/casey))

[0.20.0](https://github.com/ordinals/ord/releases/tag/0.20.0) - 2024-09-03
--------------------------------------------------------------------------

### Fixed
- Make index settings harder to misuse ([#3893](https://github.com/ordinals/ord/pull/3893) by [casey](https://github.com/casey))
- Fix rune links for runes with no symbol ([#3849](https://github.com/ordinals/ord/pull/3849) by [cryptoni9n](https://github.com/cryptoni9n))

### Added
- Add inscriptions and runes details to address API endpoint ([#3924](https://github.com/ordinals/ord/pull/3924) by [twosatsmaxi](https://github.com/twosatsmaxi))
- Add address field to `/r/inscription/:id` ([#3891](https://github.com/ordinals/ord/pull/3891) by [elocremarc](https://github.com/elocremarc))
- Add sat_balance to address API ([#3905](https://github.com/ordinals/ord/pull/3905) by [cryptoni9n](https://github.com/cryptoni9n))
- List all Bitcoin Core wallets ([#3902](https://github.com/ordinals/ord/pull/3902) by [raphjaph](https://github.com/raphjaph))

### Changed
- Remove inscription content type counts from /status page ([#3922](https://github.com/ordinals/ord/pull/3922) by [casey](https://github.com/casey))
- Suppress printing sat_ranges by default ([#3867](https://github.com/ordinals/ord/pull/3867) by [cryptoni9n](https://github.com/cryptoni9n))

### Performance
- Unified OUTPOINT_TO_UTXO_ENTRY table ([#3915](https://github.com/ordinals/ord/pull/3915) by [partialord](https://github.com/partialord))

### Misc
- Revert "Serve responses with cross origin isolation headers" ([#3920](https://github.com/ordinals/ord/pull/3920) by [casey](https://github.com/casey))
- Bump version to 0.20.0-dev ([#3916](https://github.com/ordinals/ord/pull/3916) by [casey](https://github.com/casey))
- Migrate chain.rs to snafu error ([#3904](https://github.com/ordinals/ord/pull/3904) by [cryptoni9n](https://github.com/cryptoni9n))
- Add Dutch translation to Ordinals Handbook ([#3907](https://github.com/ordinals/ord/pull/3907) by [Tibebtc](https://github.com/Tibebtc))
- Update Bitcoin Core deploy to 27.1 ([#3912](https://github.com/ordinals/ord/pull/3912) by [casey](https://github.com/casey))
- Migrate Outgoing to SnafuError ([#3854](https://github.com/ordinals/ord/pull/3854) by [cryptoni9n](https://github.com/cryptoni9n))
- Make first first and last sat in range clickable ([#3903](https://github.com/ordinals/ord/pull/3903) by [raphjaph](https://github.com/raphjaph))
- Serve responses with cross origin isolation headers ([#3898](https://github.com/ordinals/ord/pull/3898) by [patrick99e99](https://github.com/patrick99e99))
- Remove trailing space from runes specification ([#3896](https://github.com/ordinals/ord/pull/3896) by [casey](https://github.com/casey))
- Don't unnecessarily insert into utxo cache when indexing addresses ([#3894](https://github.com/ordinals/ord/pull/3894) by [raphjaph](https://github.com/raphjaph))
- Migrate object.rs to snafu error handling ([#3858](https://github.com/ordinals/ord/pull/3858) by [cryptoni9n](https://github.com/cryptoni9n))
- Clarify that unused runes tags should not be used ([#3885](https://github.com/ordinals/ord/pull/3885) by [casey](https://github.com/casey))
- Update pointer specification ([#3861](https://github.com/ordinals/ord/pull/3861) by [ansigroup](https://github.com/ansigroup))
- Re-enter beta ([#3884](https://github.com/ordinals/ord/pull/3884) by [casey](https://github.com/casey))
- Updated Chinese translation  ([#3881](https://github.com/ordinals/ord/pull/3881) by [DrJingLee](https://github.com/DrJingLee))
- Update Portuguese Translation pt.po ([#3837](https://github.com/ordinals/ord/pull/3837) by [0xArtur](https://github.com/0xArtur))

[0.19.1](https://github.com/ordinals/ord/releases/tag/0.19.1) - 2024-07-18
--------------------------------------------------------------------------

### Changed
- Commit twice to work around redb off-by-one bug ([#3856](https://github.com/ordinals/ord/pull/3856) by [casey](https://github.com/casey))

### Misc
- Change test-bitcoincore-rpc to mockcore in README.md ([#3842](https://github.com/ordinals/ord/pull/3842) by [TheHeBoy](https://github.com/TheHeBoy))

[0.19.0](https://github.com/ordinals/ord/releases/tag/0.19.0) - 2024-07-09
--------------------------------------------------------------------------

### Added
- Add inscriptions to address page ([#3843](https://github.com/ordinals/ord/pull/3843) by [raphjaph](https://github.com/raphjaph))
- Add ability to cancel shutdown ([#3820](https://github.com/ordinals/ord/pull/3820) by [felipelincoln](https://github.com/felipelincoln))
- Add charm to burned inscriptions ([#3836](https://github.com/ordinals/ord/pull/3836) by [onchainguy-btc](https://github.com/onchainguy-btc))
- Display aggregated rune balances in address page ([#3831](https://github.com/ordinals/ord/pull/3831) by [yoitsyoung](https://github.com/yoitsyoung))
- Add --all flag on `ord wallet sats` ([#3824](https://github.com/ordinals/ord/pull/3824) by [cryptoni9n](https://github.com/cryptoni9n))
- Add sat ranges to output ([#3817](https://github.com/ordinals/ord/pull/3817) by [cryptoni9n](https://github.com/cryptoni9n))
- Add sat name to inscription page ([#3826](https://github.com/ordinals/ord/pull/3826) by [cryptoni9n](https://github.com/cryptoni9n))
- Add public `shut_down()` function ([#3811](https://github.com/ordinals/ord/pull/3811) by [felipelincoln](https://github.com/felipelincoln))
- Add all transaction hex to block json response ([#3805](https://github.com/ordinals/ord/pull/3805) by [thewrlck](https://github.com/thewrlck))
- Make Index public ([#3807](https://github.com/ordinals/ord/pull/3807) by [felipelincoln](https://github.com/felipelincoln))
- Add sat balance to address page ([#3810](https://github.com/ordinals/ord/pull/3810) by [raphjaph](https://github.com/raphjaph))
- Add --http-port to settings yaml ([#3796](https://github.com/ordinals/ord/pull/3796) by [raphjaph](https://github.com/raphjaph))
- Make settings public ([#3800](https://github.com/ordinals/ord/pull/3800) by [felipelincoln](https://github.com/felipelincoln))
- Make recursive endpoints proxiable ([#3797](https://github.com/ordinals/ord/pull/3797) by [raphjaph](https://github.com/raphjaph))
- Add recursive endpoint with more details about children ([#3771](https://github.com/ordinals/ord/pull/3771) by [gmart7t2](https://github.com/gmart7t2))
- Add delegate value to recursive inscription endpoint ([#3751](https://github.com/ordinals/ord/pull/3751) by [phorkish](https://github.com/phorkish))
- Update `ord list` to include inscriptions and runes information ([#3766](https://github.com/ordinals/ord/pull/3766) by [cryptoni9n](https://github.com/cryptoni9n))
- Add feerate percentiles to blockinfo endpoint ([#3753](https://github.com/ordinals/ord/pull/3753) by [benbuschmann](https://github.com/benbuschmann))
- Add mint progress field to rune.html ([#3748](https://github.com/ordinals/ord/pull/3748) by [cryptoni9n](https://github.com/cryptoni9n))
- Add /inscription/:query/:child_number route ([#3777](https://github.com/ordinals/ord/pull/3777) by [casey](https://github.com/casey))
- Add parents recursive endpoint ([#3749](https://github.com/ordinals/ord/pull/3749) by [phorkish](https://github.com/phorkish))
- Index addresses ([#3757](https://github.com/ordinals/ord/pull/3757) by [raphjaph](https://github.com/raphjaph))
- Make settings public ([#3768](https://github.com/ordinals/ord/pull/3768) by [raphjaph](https://github.com/raphjaph))
- Add decode api ([#3733](https://github.com/ordinals/ord/pull/3733) by [shadowv0vshadow](https://github.com/shadowv0vshadow))
- Add command to list pending etchings ([#3732](https://github.com/ordinals/ord/pull/3732) by [ldiego08](https://github.com/ldiego08))
- Add `ord wallet runics` command ([#3734](https://github.com/ordinals/ord/pull/3734) by [ldiego08](https://github.com/ldiego08))

### Changed
- Enable resuming a specific rune etching ([#3679](https://github.com/ordinals/ord/pull/3679) by [ldiego08](https://github.com/ldiego08))

### Fixed
- Fix panic in ord env shutdown ([#3787](https://github.com/ordinals/ord/pull/3787) by [cryptoni9n](https://github.com/cryptoni9n))
- Allow postage equal to dust limit in mint.rs ([#3756](https://github.com/ordinals/ord/pull/3756) by [gmart7t2](https://github.com/gmart7t2))
- Update index.rs to fix ord list command ([#3762](https://github.com/ordinals/ord/pull/3762) by [cryptoni9n](https://github.com/cryptoni9n))

### Misc
- Update Spanish Translation ([#3835](https://github.com/ordinals/ord/pull/3835) by [Zerone495](https://github.com/Zerone495))
- Add debugging tips README ([#3823](https://github.com/ordinals/ord/pull/3823) by [nick07002](https://github.com/nick07002))
- Add typed errors with `snafu` ([#3832](https://github.com/ordinals/ord/pull/3832) by [casey](https://github.com/casey))
- Add -dev suffix to version ([#3812](https://github.com/ordinals/ord/pull/3812) by [casey](https://github.com/casey))
- Link address on output & tx ([#3799](https://github.com/ordinals/ord/pull/3799) by [cryptoni9n](https://github.com/cryptoni9n))
- Link address on inscription.html ([#3801](https://github.com/ordinals/ord/pull/3801) by [cryptoni9n](https://github.com/cryptoni9n))
- Fix fuzz testers  ([#3740](https://github.com/ordinals/ord/pull/3740) by [jeasonstudio](https://github.com/jeasonstudio))
- Remove duplicate example ([#3776](https://github.com/ordinals/ord/pull/3776) by [gmart7t2](https://github.com/gmart7t2))
- Update runes spec ([#3745](https://github.com/ordinals/ord/pull/3745) by [gmart7t2](https://github.com/gmart7t2))
- Clarify teleburning.md ([#3744](https://github.com/ordinals/ord/pull/3744) by [gmart7t2](https://github.com/gmart7t2))

[0.18.5](https://github.com/ordinals/ord/releases/tag/0.18.5) - 2024-05-09
--------------------------------------------------------------------------

### Added
- Allow specifying different output formats ([#3424](https://github.com/ordinals/ord/pull/3424) by [bingryan](https://github.com/bingryan))
- Allow higher rpcworkqueue limit conf ([#3615](https://github.com/ordinals/ord/pull/3615) by [JeremyRubin](https://github.com/JeremyRubin))
- Show progress bar for etching ([#3673](https://github.com/ordinals/ord/pull/3673) by [twosatsmaxi](https://github.com/twosatsmaxi))

### Fixed
- Update sat-hunting.md ([#3724](https://github.com/ordinals/ord/pull/3724) by [cryptoni9n](https://github.com/cryptoni9n))
- Update runes.md docs ([#3681](https://github.com/ordinals/ord/pull/3681) by [hantuzun](https://github.com/hantuzun))
- Patch some omissions in the Chinese translation ([#3694](https://github.com/ordinals/ord/pull/3694) by [shadowv0vshadow](https://github.com/shadowv0vshadow))
- Bump rustfmt version 2018 to 2021 ([#3721](https://github.com/ordinals/ord/pull/3721) by [bingryan](https://github.com/bingryan))

[0.18.4](https://github.com/ordinals/ord/releases/tag/0.18.4) - 2024-05-02
--------------------------------------------------------------------------

### Added
- Clarify that inscriptions must be served from URLs with path /content/<INSCRIPTION_ID> ([#3209](https://github.com/ordinals/ord/pull/3209) by [Vanniix](https://github.com/Vanniix))

### Changed
- Persist config files for ord env command ([#3715](https://github.com/ordinals/ord/pull/3715) by [twosatsmaxi](https://github.com/twosatsmaxi))
- Do not show runic outputs in cardinals command ([#3656](https://github.com/ordinals/ord/pull/3656) by [raphjaph](https://github.com/raphjaph))

### Fixed
- Fix send runes ([#3484](https://github.com/ordinals/ord/pull/3484) by [raphjaph](https://github.com/raphjaph))
- Allow longer request body for JSON API ([#3655](https://github.com/ordinals/ord/pull/3655) by [raphjaph](https://github.com/raphjaph))
- Allow minting if mint begins next block ([#3659](https://github.com/ordinals/ord/pull/3659) by [casey](https://github.com/casey))

### Misc
- Add alt text to preview image ([#3713](https://github.com/ordinals/ord/pull/3713) by [losingle](https://github.com/losingle))
- Remove duplicate endpoint from explorer.md ([#3716](https://github.com/ordinals/ord/pull/3716) by [cryptoni9n](https://github.com/cryptoni9n))
- Use correct content type for .mjs inscriptions ([#3712](https://github.com/ordinals/ord/pull/3712) by [casey](https://github.com/casey))
- Add support for mjs files ([#3653](https://github.com/ordinals/ord/pull/3653) by [elocremarc](https://github.com/elocremarc))
- Fix typo on sat hunting page ([#3668](https://github.com/ordinals/ord/pull/3668) by [cryptoni9n](https://github.com/cryptoni9n))
- Use contains_key instead of get / is_some ([#3705](https://github.com/ordinals/ord/pull/3705) by [knowmost](https://github.com/knowmost))
- Update sat-hunting.md with how to transfer specific sats ([#3666](https://github.com/ordinals/ord/pull/3666) by [cryptoni9n](https://github.com/cryptoni9n))
- Fix zh.po translations ([#3588](https://github.com/ordinals/ord/pull/3588) by [losingle](https://github.com/losingle))
- Update sparrow-wallet.md --name flag update ([#3635](https://github.com/ordinals/ord/pull/3635) by [taha-abbasi](https://github.com/taha-abbasi))

[0.18.3](https://github.com/ordinals/ord/releases/tag/0.18.3) - 2024-04-19
--------------------------------------------------------------------------

### Added
- Add `dry-run` flag to `resume` command ([#3592](https://github.com/ordinals/ord/pull/3592) by [felipelincoln](https://github.com/felipelincoln))
- Add back runes balances API ([#3571](https://github.com/ordinals/ord/pull/3571) by [lugondev](https://github.com/lugondev))
- Show premine percentage ([#3567](https://github.com/ordinals/ord/pull/3567) by [raphjaph](https://github.com/raphjaph))
- Add default content proxy and decompress to env command ([#3509](https://github.com/ordinals/ord/pull/3509) by [jahvi](https://github.com/jahvi))

### Changed
- Resume cycles through all pending etchings ([#3566](https://github.com/ordinals/ord/pull/3566) by [raphjaph](https://github.com/raphjaph))

### Misc
- Check rune minimum at height before sending ([#3626](https://github.com/ordinals/ord/pull/3626) by [raphjaph](https://github.com/raphjaph))
- Update recursion.md with consistant syntax ([#3585](https://github.com/ordinals/ord/pull/3585) by [zmeyer44](https://github.com/zmeyer44))
- Add test Rune cannot be minted less than limit amount ([#3556](https://github.com/ordinals/ord/pull/3556) by [lugondev](https://github.com/lugondev))
- Clear etching when rune commitment is spent ([#3618](https://github.com/ordinals/ord/pull/3618) by [felipelincoln](https://github.com/felipelincoln))
- Remove timeout for wallet client ([#3621](https://github.com/ordinals/ord/pull/3621) by [raphjaph](https://github.com/raphjaph))
- Remove duplicated word ([#3598](https://github.com/ordinals/ord/pull/3598) by [oxSaturn](https://github.com/oxSaturn))
- Address runes review comments ([#3605](https://github.com/ordinals/ord/pull/3605) by [casey](https://github.com/casey))
- Generate sample batch.yaml in env command ([#3530](https://github.com/ordinals/ord/pull/3530) by [twosatsmaxi](https://github.com/twosatsmaxi))

[0.18.2](https://github.com/ordinals/ord/releases/tag/0.18.2) - 2024-04-17
--------------------------------------------------------------------------

### Migration
- Wallet databases are now stored in the `/wallets` subdirectory of the data
  dir. To use old wallet databases with 0.18.2, move `<WALLET_NAME>.redb` files
  into the `/wallets` subdirectory of the data dir. Currently, the only
  information stored in wallet databases are pending etchings.

### Changed
- Store wallets in /wallets subdir of data dir ([#3553](https://github.com/ordinals/ord/pull/3553) by [casey](https://github.com/casey))
- Remove /runes/balances page ([#3555](https://github.com/ordinals/ord/pull/3555) by [lugondev](https://github.com/lugondev))
- Forbid etching below rune activation height ([#3523](https://github.com/ordinals/ord/pull/3523) by [casey](https://github.com/casey))

### Added
- Add command to export BIP-329 labels for wallet outputs ([#3120](https://github.com/ordinals/ord/pull/3120) by [casey](https://github.com/casey))
- Display etched runes on /block ([#3366](https://github.com/ordinals/ord/pull/3366) by [lugondev](https://github.com/lugondev))
- Emit rune-related events ([#3219](https://github.com/ordinals/ord/pull/3219) by [felipelincoln](https://github.com/felipelincoln))
- Lookup rune by number ([#3440](https://github.com/ordinals/ord/pull/3440) by [lugondev](https://github.com/lugondev))
- Add runes pagination ([#3215](https://github.com/ordinals/ord/pull/3215) by [lugondev](https://github.com/lugondev))

### Misc
- Document turbo flag ([#3579](https://github.com/ordinals/ord/pull/3579) by [gmart7t2](https://github.com/gmart7t2))
- Add open mint tests ([#3558](https://github.com/ordinals/ord/pull/3558) by [lugondev](https://github.com/lugondev))
- Fix typos ([#3541](https://github.com/ordinals/ord/pull/3541) by [StevenMia](https://github.com/StevenMia))
- Fix typo in zh.po ([#3540](https://github.com/ordinals/ord/pull/3540) by [blackj-x](https://github.com/blackj-x))
- Lock runes commit output ([#3504](https://github.com/ordinals/ord/pull/3504) by [raphjaph](https://github.com/raphjaph))
- Address runes review comments ([#3547](https://github.com/ordinals/ord/pull/3547) by [casey](https://github.com/casey))
- Add Red Had build instructions to readme ([#3531](https://github.com/ordinals/ord/pull/3531) by [rongyi](https://github.com/rongyi))
- Fix typo in recursion docs ([#3529](https://github.com/ordinals/ord/pull/3529) by [nix-eth](https://github.com/nix-eth))
- Put rune higher on /inscription ([#3363](https://github.com/ordinals/ord/pull/3363) by [lugondev](https://github.com/lugondev))

[0.18.1](https://github.com/ordinals/ord/releases/tag/0.18.1) - 2024-04-11
--------------------------------------------------------------------------

### Fixed
- Fix off-by-one in wallet when waiting for etching commitment to mature ([#3515](https://github.com/ordinals/ord/pull/3515) by [casey](https://github.com/casey))

[0.18.0](https://github.com/ordinals/ord/releases/tag/0.18.0) - 2024-04-10
--------------------------------------------------------------------------

### Fixed
- Check etching commit confirmations correctly ([#3507](https://github.com/ordinals/ord/pull/3507) by [casey](https://github.com/casey))

### Added
- Add postage flag to mint command ([#3482](https://github.com/ordinals/ord/pull/3482) by [ynohtna92](https://github.com/ynohtna92))
- Mint with destination ([#3497](https://github.com/ordinals/ord/pull/3497) by [ynohtna92](https://github.com/ynohtna92))
- Add etching turbo flag ([#3511](https://github.com/ordinals/ord/pull/3511) by [casey](https://github.com/casey))
- Allow inscribing without file ([#3451](https://github.com/ordinals/ord/pull/3451) by [raphjaph](https://github.com/raphjaph))
- Add wallet batch outputs and inscriptions endpoints ([#3456](https://github.com/ordinals/ord/pull/3456) by [raphjaph](https://github.com/raphjaph))

### Changed
- Show decimal rune balances ([#3505](https://github.com/ordinals/ord/pull/3505) by [raphjaph](https://github.com/raphjaph))

### Misc
- Test that mints without a cap are unmintable ([#3495](https://github.com/ordinals/ord/pull/3495) by [lugondev](https://github.com/lugondev))
- Bump ord crate required rust version to 1.76 ([#3512](https://github.com/ordinals/ord/pull/3512) by [casey](https://github.com/casey))
- Updated rust-version to 1.74.0 ([#3492](https://github.com/ordinals/ord/pull/3492) by [tgscan-dev](https://github.com/tgscan-dev))
- Better error message when bitcoind doesn't start ([#3500](https://github.com/ordinals/ord/pull/3500) by [twosatsmaxi](https://github.com/twosatsmaxi))
- Fix typo in zh.po ([#3498](https://github.com/ordinals/ord/pull/3498) by [RandolphJiffy](https://github.com/RandolphJiffy))
- Document allowed opcodes in runestones ([#3461](https://github.com/ordinals/ord/pull/3461) by [casey](https://github.com/casey))
- Update data carriersize to match with ord ([#3506](https://github.com/ordinals/ord/pull/3506) by [twosatsmaxi](https://github.com/twosatsmaxi))
- Fix maturation loop ([#3480](https://github.com/ordinals/ord/pull/3480) by [raphjaph](https://github.com/raphjaph))
- Add rune logo and link to navbar ([#3442](https://github.com/ordinals/ord/pull/3442) by [lugondev](https://github.com/lugondev))
- Add package necessary for Ubuntu ([#3462](https://github.com/ordinals/ord/pull/3462) by [petriuslima](https://github.com/petriuslima))
- Update required Rust version in README ([#3466](https://github.com/ordinals/ord/pull/3466) by [cryptoni9n](https://github.com/cryptoni9n))
- Fix typo in zh.po ([#3464](https://github.com/ordinals/ord/pull/3464) by [RandolphJiffy](https://github.com/RandolphJiffy))
- Update testing.md ([#3463](https://github.com/ordinals/ord/pull/3463) by [gmart7t2](https://github.com/gmart7t2))
- Update rune docs for Chinese version ([#3457](https://github.com/ordinals/ord/pull/3457) by [DrJingLee](https://github.com/DrJingLee))
- Remove `etch` from error message ([#3449](https://github.com/ordinals/ord/pull/3449) by [ordinariusprof](https://github.com/ordinariusprof))
- Fix deploy bitcoin.conf typo ([#3443](https://github.com/ordinals/ord/pull/3443) by [bitspill](https://github.com/bitspill))
- Fix type in runes docs ([#3447](https://github.com/ordinals/ord/pull/3447) by [twosatsmaxi](https://github.com/twosatsmaxi))

[0.17.1](https://github.com/ordinals/ord/releases/tag/0.17.1) - 2024-04-01
--------------------------------------------------------------------------

### Fixed
- Ignore invalid script pubkeys ([#3432](https://github.com/ordinals/ord/pull/3432) by [casey](https://github.com/casey))

### Misc
- Fix typo ([#3429](https://github.com/ordinals/ord/pull/3429) by [lugondev](https://github.com/lugondev))
- Relax deployed Bitcoin Core relay rules ([#3431](https://github.com/ordinals/ord/pull/3431) by [casey](https://github.com/casey))

[0.17.0](https://github.com/ordinals/ord/releases/tag/0.17.0) - 2024-03-31
--------------------------------------------------------------------------

### Added
- Allow pausing and resuming etchings ([#3374](https://github.com/ordinals/ord/pull/3374) by [raphjaph](https://github.com/raphjaph))
- Seed index with genesis rune ([#3426](https://github.com/ordinals/ord/pull/3426) by [casey](https://github.com/casey))
- Add `ord wallet batch` command ([#3401](https://github.com/ordinals/ord/pull/3401) by [casey](https://github.com/casey))
- Return effective content type in JSON API ([#3289](https://github.com/ordinals/ord/pull/3289) by [arik-so](https://github.com/arik-so))
- Mint terms ([#3375](https://github.com/ordinals/ord/pull/3375) by [casey](https://github.com/casey))
- Allow supply-capped mints ([#3365](https://github.com/ordinals/ord/pull/3365) by [casey](https://github.com/casey))
- Return runestone from `ord decode` ([#3349](https://github.com/ordinals/ord/pull/3349) by [casey](https://github.com/casey))
- Display charms on /sat ([#3340](https://github.com/ordinals/ord/pull/3340) by [markovichecha](https://github.com/markovichecha))
- Allow sending sat ([#3200](https://github.com/ordinals/ord/pull/3200) by [bingryan](https://github.com/bingryan))
- Display mintability on /rune ([#3324](https://github.com/ordinals/ord/pull/3324) by [raphjaph](https://github.com/raphjaph))
- Mint runes with wallet ([#3298](https://github.com/ordinals/ord/pull/3298) by [raphjaph](https://github.com/raphjaph))
- Index multiple parents ([#3227](https://github.com/ordinals/ord/pull/3227) by [arik-so](https://github.com/arik-so))
- Add fallback route ([#3288](https://github.com/ordinals/ord/pull/3288) by [casey](https://github.com/casey))
- Allow looking up inscriptions by sat name ([#3286](https://github.com/ordinals/ord/pull/3286) by [casey](https://github.com/casey))
- Allow generating multiple receive addresses ([#3277](https://github.com/ordinals/ord/pull/3277) by [bingryan](https://github.com/bingryan))

### Changed
- Recognized field without required flag produce cenotaphs ([#3422](https://github.com/ordinals/ord/pull/3422) by [casey](https://github.com/casey))
- Rename test-bitcoincore-rpc to mockcore ([#3415](https://github.com/ordinals/ord/pull/3415) by [casey](https://github.com/casey))
- Derive reserved rune names from rune ID ([#3412](https://github.com/ordinals/ord/pull/3412) by [casey](https://github.com/casey))
- Don't complain about large runestones if --no-limit is passed ([#3402](https://github.com/ordinals/ord/pull/3402) by [casey](https://github.com/casey))
- Move runes types into ordinals crate ([#3391](https://github.com/ordinals/ord/pull/3391) by [casey](https://github.com/casey))
- Disambiguate when sending runes ([#3368](https://github.com/ordinals/ord/pull/3368) by [raphjaph](https://github.com/raphjaph))
- Only allow sending sats by name ([#3344](https://github.com/ordinals/ord/pull/3344) by [casey](https://github.com/casey))
- Downgrade from `beta` to `alpha` ([#3315](https://github.com/ordinals/ord/pull/3315) by [casey](https://github.com/casey))

### Misc
- Add links to status page ([#3361](https://github.com/ordinals/ord/pull/3361) by [lugondev](https://github.com/lugondev))
- Document sending runes ([#3405](https://github.com/ordinals/ord/pull/3405) by [rot13maxi](https://github.com/rot13maxi))
- Use checked arithmetic in RuneUpdater ([#3423](https://github.com/ordinals/ord/pull/3423) by [casey](https://github.com/casey))
- Update Dockerfile Rust version ([#3425](https://github.com/ordinals/ord/pull/3425) by [0xspyop](https://github.com/0xspyop))
- Don't conflate cenotaphs and runestones ([#3417](https://github.com/ordinals/ord/pull/3417) by [casey](https://github.com/casey))
- Fix typos ([#3418](https://github.com/ordinals/ord/pull/3418) by [xiaoxianBoy](https://github.com/xiaoxianBoy))
- Set pointer in etching runestone ([#3420](https://github.com/ordinals/ord/pull/3420) by [casey](https://github.com/casey))
- Fix fuzz tests ([#3416](https://github.com/ordinals/ord/pull/3416) by [casey](https://github.com/casey))
- Set relative lock height on etching transactions ([#3414](https://github.com/ordinals/ord/pull/3414) by [casey](https://github.com/casey))
- Add CTRL-C test ([#3413](https://github.com/ordinals/ord/pull/3413) by [raphjaph](https://github.com/raphjaph))
- Add etching to example batchfile ([#3407](https://github.com/ordinals/ord/pull/3407) by [casey](https://github.com/casey))
- Fix inscribe_with_no_limit test ([#3403](https://github.com/ordinals/ord/pull/3403) by [casey](https://github.com/casey))
- Rename Inscribe to Batch in integration tests ([#3404](https://github.com/ordinals/ord/pull/3404) by [casey](https://github.com/casey))
- Distinguish invalid opcode and invalid script ([#3400](https://github.com/ordinals/ord/pull/3400) by [casey](https://github.com/casey))
- Fix rune ID delta-encoding table ([#3393](https://github.com/ordinals/ord/pull/3393) by [chendatony31](https://github.com/chendatony31))
- Handle invalid scripts correctly ([#3390](https://github.com/ordinals/ord/pull/3390) by [casey](https://github.com/casey))
- Fix typo: Eching -> Etching ([#3397](https://github.com/ordinals/ord/pull/3397) by [gmart7t2](https://github.com/gmart7t2))
- Fix typo: transactions -> transaction's ([#3398](https://github.com/ordinals/ord/pull/3398) by [gmart7t2](https://github.com/gmart7t2))
- Fix typo: an -> a ([#3395](https://github.com/ordinals/ord/pull/3395) by [gmart7t2](https://github.com/gmart7t2))
- Fix runes docs table ([#3389](https://github.com/ordinals/ord/pull/3389) by [casey](https://github.com/casey))
- Document runes ([#3380](https://github.com/ordinals/ord/pull/3380) by [casey](https://github.com/casey))
- Check mint runestone ([#3388](https://github.com/ordinals/ord/pull/3388) by [casey](https://github.com/casey))
- Check send runestone ([#3386](https://github.com/ordinals/ord/pull/3386) by [casey](https://github.com/casey))
- Decimal::to_amount → Decimal::to_integer ([#3382](https://github.com/ordinals/ord/pull/3382) by [casey](https://github.com/casey))
- Add SpacedRune test case ([#3379](https://github.com/ordinals/ord/pull/3379) by [casey](https://github.com/casey))
- Add Runestone::cenotaph() ([#3381](https://github.com/ordinals/ord/pull/3381) by [casey](https://github.com/casey))
- Terms::limit → Terms::amount ([#3383](https://github.com/ordinals/ord/pull/3383) by [casey](https://github.com/casey))
- Use default() as shorthand for Default::default() ([#3371](https://github.com/ordinals/ord/pull/3371) by [casey](https://github.com/casey))
- Add batch module to wallet ([#3359](https://github.com/ordinals/ord/pull/3359) by [casey](https://github.com/casey))
- Make rune parent clickable ([#3358](https://github.com/ordinals/ord/pull/3358) by [raphjaph](https://github.com/raphjaph))
- Assert etched runestone is correct ([#3354](https://github.com/ordinals/ord/pull/3354) by [casey](https://github.com/casey))
- Display spaced runes in balances ([#3353](https://github.com/ordinals/ord/pull/3353) by [casey](https://github.com/casey))
- Cleanup ([#3348](https://github.com/ordinals/ord/pull/3348) by [lugondev](https://github.com/lugondev))
- Fetch etching inputs using Bitcoin Core RPC ([#3336](https://github.com/ordinals/ord/pull/3336) by [raphjaph](https://github.com/raphjaph))
- Update Chinese version of handbook ([#3334](https://github.com/ordinals/ord/pull/3334) by [DrJingLee](https://github.com/DrJingLee))
- Use serde_with::DeserializeFromStr ([#3343](https://github.com/ordinals/ord/pull/3343) by [casey](https://github.com/casey))
- Remove quotes from example ord env command ([#3335](https://github.com/ordinals/ord/pull/3335) by [casey](https://github.com/casey))
- Initial runes review ([#3331](https://github.com/ordinals/ord/pull/3331) by [casey](https://github.com/casey))
- Fix redundant locking ([#3342](https://github.com/ordinals/ord/pull/3342) by [raphjaph](https://github.com/raphjaph))
- Derive Deserialize for Runestone ([#3339](https://github.com/ordinals/ord/pull/3339) by [emilcondrea](https://github.com/emilcondrea))
- Update redb to 2.0.0 ([#3341](https://github.com/ordinals/ord/pull/3341) by [cberner](https://github.com/cberner))
- Runestones with unknown semantics are cenotaphs ([#3325](https://github.com/ordinals/ord/pull/3325) by [casey](https://github.com/casey))
- Reserve rune IDs with zero block and nonzero tx ([#3323](https://github.com/ordinals/ord/pull/3323) by [casey](https://github.com/casey))
- Display rune premine ([#3313](https://github.com/ordinals/ord/pull/3313) by [raphjaph](https://github.com/raphjaph))
- Make max mint limit u64::MAX ([#3316](https://github.com/ordinals/ord/pull/3316) by [casey](https://github.com/casey))
- Change rune protocol identifier to OP_PUSHNUM_13 ([#3314](https://github.com/ordinals/ord/pull/3314) by [casey](https://github.com/casey))
- Strict edicts ([#3312](https://github.com/ordinals/ord/pull/3312) by [casey](https://github.com/casey))
- Allow premining with open etchings ([#3311](https://github.com/ordinals/ord/pull/3311) by [raphjaph](https://github.com/raphjaph))
- Rename RuneID fields ([#3310](https://github.com/ordinals/ord/pull/3310) by [casey](https://github.com/casey))
- Prevent front-running rune etchings ([#3212](https://github.com/ordinals/ord/pull/3212) by [casey](https://github.com/casey))
- Clarify build instructions ([#3304](https://github.com/ordinals/ord/pull/3304) by [raphjaph](https://github.com/raphjaph))
- Add test to choose the earliest of deadline or end ([#3254](https://github.com/ordinals/ord/pull/3254) by [sondotpin](https://github.com/sondotpin))
- Ensure inscription tags are unique ([#3296](https://github.com/ordinals/ord/pull/3296) by [casey](https://github.com/casey))
- Include CSP origin in preview content security policy headers ([#3276](https://github.com/ordinals/ord/pull/3276) by [bingryan](https://github.com/bingryan))
- Add pre-commit hook ([#3262](https://github.com/ordinals/ord/pull/3262) by [bingryan](https://github.com/bingryan))
- Fix querying for inscriptions by sat names containing `i` ([#3287](https://github.com/ordinals/ord/pull/3287) by [casey](https://github.com/casey))
- Switch recommended flag usage from `--data-dir` to `--datadir` ([#3281](https://github.com/ordinals/ord/pull/3281) by [chasefleming](https://github.com/chasefleming))
- Better wallet error message ([#3272](https://github.com/ordinals/ord/pull/3272) by [bingryan](https://github.com/bingryan))
- Add recipe to delete indices ([#3266](https://github.com/ordinals/ord/pull/3266) by [casey](https://github.com/casey))
- Bump ordinals version: 0.0.3 → 0.0.4 ([#3267](https://github.com/ordinals/ord/pull/3267) by [casey](https://github.com/casey))

[0.16.0](https://github.com/ordinals/ord/releases/tag/0.16.0) - 2024-03-11
--------------------------------------------------------------------------

### Added
- Document recursive endpoint backwards compatibility guarantees ([#3265](https://github.com/ordinals/ord/pull/3265) by [casey](https://github.com/casey))
- Reserve inscription tag 15 ([#3256](https://github.com/ordinals/ord/pull/3256) by [casey](https://github.com/casey))
- Display initial sync time on status page ([#3250](https://github.com/ordinals/ord/pull/3250) by [casey](https://github.com/casey))
- Add content proxy ([#3216](https://github.com/ordinals/ord/pull/3216) by [raphjaph](https://github.com/raphjaph))
- Allow configuring interval between commits to index ([#3186](https://github.com/ordinals/ord/pull/3186) by [bingryan](https://github.com/bingryan))
- Print PSBT for dry run inscribe ([#3116](https://github.com/ordinals/ord/pull/3116) by [raphjaph](https://github.com/raphjaph))
- Add parent preview to inscription page ([#3163](https://github.com/ordinals/ord/pull/3163) by [elocremarc](https://github.com/elocremarc))
- Add `/r/inscription` endpoint for getting inscription details ([#2628](https://github.com/ordinals/ord/pull/2628) by [devords](https://github.com/devords))
- Add optional HTTP authentication for server ([#3131](https://github.com/ordinals/ord/pull/3131) by [casey](https://github.com/casey))
- Display inscription content type counts on /status ([#3127](https://github.com/ordinals/ord/pull/3127) by [casey](https://github.com/casey))
- Add `ord env` to spin up a test bitcoin daemon and ord server ([#3146](https://github.com/ordinals/ord/pull/3146) by [casey](https://github.com/casey))
- Emit inscription update events to channel ([#3137](https://github.com/ordinals/ord/pull/3137) by [mi-yu](https://github.com/mi-yu))
- Allow inscribing AVIF images ([#3123](https://github.com/ordinals/ord/pull/3123) by [casey](https://github.com/casey))
- Add `satpoints` batch inscribe mode ([#3115](https://github.com/ordinals/ord/pull/3115) by [raphjaph](https://github.com/raphjaph))
- Add /r/blockinfo endpoint ([#3075](https://github.com/ordinals/ord/pull/3075) by [jerryfane](https://github.com/jerryfane))
- Return signed PSBT from `ord wallet send` ([#3093](https://github.com/ordinals/ord/pull/3093) by [raphjaph](https://github.com/raphjaph))
- Add /runes/balances ([#2978](https://github.com/ordinals/ord/pull/2978) by [lugondev](https://github.com/lugondev))
- Dump and restore wallet from descriptors ([#3048](https://github.com/ordinals/ord/pull/3048) by [raphjaph](https://github.com/raphjaph))
- Inscribe with delegate ([#3021](https://github.com/ordinals/ord/pull/3021) by [casey](https://github.com/casey))
- Add option to retain sat index for spent outputs ([#2999](https://github.com/ordinals/ord/pull/2999) by [casey](https://github.com/casey))
- Add `indexed` to output JSON ([#2971](https://github.com/ordinals/ord/pull/2971) by [raphjaph](https://github.com/raphjaph))

### Changed
- Add `id` inscription recursive JSON ([#3258](https://github.com/ordinals/ord/pull/3258) by [raphjaph](https://github.com/raphjaph))
- Add more fields to /r/blockinfo ([#3260](https://github.com/ordinals/ord/pull/3260) by [raphjaph](https://github.com/raphjaph))
- Load config from default data dir and configure `ord env ` using config ([#3240](https://github.com/ordinals/ord/pull/3240) by [casey](https://github.com/casey))
- Overhaul settings ([#3188](https://github.com/ordinals/ord/pull/3188) by [casey](https://github.com/casey))
- Improve configuration ([#3156](https://github.com/ordinals/ord/pull/3156) by [casey](https://github.com/casey))
- Represent rune IDs as `BLOCK:TX` ([#3165](https://github.com/ordinals/ord/pull/3165) by [casey](https://github.com/casey))
- Display parent above metadata on /inscription ([#3160](https://github.com/ordinals/ord/pull/3160) by [casey](https://github.com/casey))
- Make `ord env` more user friendly ([#3153](https://github.com/ordinals/ord/pull/3153) by [casey](https://github.com/casey))
- Use `image-rendering: auto` for AVIF inscriptions ([#3148](https://github.com/ordinals/ord/pull/3148) by [casey](https://github.com/casey))
- Make wallet async ([#3142](https://github.com/ordinals/ord/pull/3142) by [raphjaph](https://github.com/raphjaph))
- Use `image-rendering: auto` when downscaling images ([#3144](https://github.com/ordinals/ord/pull/3144) by [casey](https://github.com/casey))
- Only allow mnemonic from stdin ([#3023](https://github.com/ordinals/ord/pull/3023) by [mj10021](https://github.com/mj10021))
- Show reinscriptions in `ord wallet inscriptions` ([#3101](https://github.com/ordinals/ord/pull/3101) by [raphjaph](https://github.com/raphjaph))
- Allow specifying satpoint in `same-sat` batch inscribe ([#3100](https://github.com/ordinals/ord/pull/3100) by [raphjaph](https://github.com/raphjaph))
- Enable JSON API by default ([#3047](https://github.com/ordinals/ord/pull/3047) by [raphjaph](https://github.com/raphjaph))
- Make wallet communicate with index via RPC ([#2929](https://github.com/ordinals/ord/pull/2929) by [raphjaph](https://github.com/raphjaph))
- Add blocks and transaction JSON endpoints ([#3004](https://github.com/ordinals/ord/pull/3004) by [DaviRain-Su](https://github.com/DaviRain-Su))
- Hide BVM Network inscriptions ([#3012](https://github.com/ordinals/ord/pull/3012) by [casey](https://github.com/casey))
- Suppress empty command output ([#2995](https://github.com/ordinals/ord/pull/2995) by [casey](https://github.com/casey))

### Misc
- Rename genesis fee to inscription fee ([#3257](https://github.com/ordinals/ord/pull/3257) by [raphjaph](https://github.com/raphjaph))
- Don't consider unconfirmed UTXOs as spent ([#3255](https://github.com/ordinals/ord/pull/3255) by [arik-so](https://github.com/arik-so))
- Create tempdir in download-log recipe ([#3242](https://github.com/ordinals/ord/pull/3242) by [casey](https://github.com/casey))
- Fix list numbering in handbook ([#3248](https://github.com/ordinals/ord/pull/3248) by [lugondev](https://github.com/lugondev))
- Document `ord env` commands ([#3241](https://github.com/ordinals/ord/pull/3241) by [casey](https://github.com/casey))
- Document `ord wallet restore` ([#3237](https://github.com/ordinals/ord/pull/3237) by [raphjaph](https://github.com/raphjaph))
- Enable indexing runes on mainnet ([#3236](https://github.com/ordinals/ord/pull/3236) by [casey](https://github.com/casey))
- Add libssl-dev to ubuntu install command ([#3235](https://github.com/ordinals/ord/pull/3235) by [andrewhong5297](https://github.com/andrewhong5297))
- Test that runes can be minted with no edict ([#3231](https://github.com/ordinals/ord/pull/3231) by [casey](https://github.com/casey))
- Rename index_envelopes to index_inscriptions ([#3233](https://github.com/ordinals/ord/pull/3233) by [casey](https://github.com/casey))
- Check for duplicate satpoints in `satpoints` mode ([#3221](https://github.com/ordinals/ord/pull/3221) by [raphjaph](https://github.com/raphjaph))
- Add reinscribe option to batch file ([#3220](https://github.com/ordinals/ord/pull/3220) by [raphjaph](https://github.com/raphjaph))
- Encode claims as tag ([#3206](https://github.com/ordinals/ord/pull/3206) by [casey](https://github.com/casey))
- Make nop and burn tags one byte ([#3207](https://github.com/ordinals/ord/pull/3207) by [casey](https://github.com/casey))
- Make deploys noninteractive ([#3189](https://github.com/ordinals/ord/pull/3189) by [casey](https://github.com/casey))
- Credit contributors in changelog ([#3187](https://github.com/ordinals/ord/pull/3187) by [casey](https://github.com/casey))
- Update ordinals crate ([#3184](https://github.com/ordinals/ord/pull/3184) by [raphjaph](https://github.com/raphjaph))
- Refactor test server to use arguments ([#3183](https://github.com/ordinals/ord/pull/3183) by [casey](https://github.com/casey))
- Install openssl in docker image ([#3181](https://github.com/ordinals/ord/pull/3181) by [aekasitt](https://github.com/aekasitt))
- Document `ord env` ([#3180](https://github.com/ordinals/ord/pull/3180) by [casey](https://github.com/casey))
- Update docs to reflect wallet changes ([#3179](https://github.com/ordinals/ord/pull/3179) by [raphjaph](https://github.com/raphjaph))
- Remove unnecessary lifetime from Formatter ([#3178](https://github.com/ordinals/ord/pull/3178) by [casey](https://github.com/casey))
- Fix lints ([#3124](https://github.com/ordinals/ord/pull/3124) by [lugondev](https://github.com/lugondev))
- Update inscription sat documentation ([#3114](https://github.com/ordinals/ord/pull/3114) by [zhiqiangxu](https://github.com/zhiqiangxu))
- Move JSON structs into api module ([#3167](https://github.com/ordinals/ord/pull/3167) by [casey](https://github.com/casey))
- Make Options public ([#3138](https://github.com/ordinals/ord/pull/3138) by [mi-yu](https://github.com/mi-yu))
- Fix spelling mistake in bip.mediawiki ([#3118](https://github.com/ordinals/ord/pull/3118) by [HarveyV](https://github.com/HarveyV))
- Import multiple descriptors at a time ([#3091](https://github.com/ordinals/ord/pull/3091) by [raphjaph](https://github.com/raphjaph))
- fix naming ([#3112](https://github.com/ordinals/ord/pull/3112) by [zhiqiangxu](https://github.com/zhiqiangxu))
- Move sat and friends into ordinals crate ([#3079](https://github.com/ordinals/ord/pull/3079) by [raphjaph](https://github.com/raphjaph))
- Remove index parameter from index_block ([#3088](https://github.com/ordinals/ord/pull/3088) by [zhiqiangxu](https://github.com/zhiqiangxu))
- Make clippy stop complaining about insane repair callback ([#3104](https://github.com/ordinals/ord/pull/3104) by [casey](https://github.com/casey))
- Use min instead of clamp ([#3081](https://github.com/ordinals/ord/pull/3081) by [zhiqiangxu](https://github.com/zhiqiangxu))
- [ordinals] Bump version: 0.0.1 → 0.0.2 ([#3078](https://github.com/ordinals/ord/pull/3078) by [casey](https://github.com/casey))
- Move SatPoint into library ([#3077](https://github.com/ordinals/ord/pull/3077) by [casey](https://github.com/casey))
- Use a flag to indicate a mint ([#3068](https://github.com/ordinals/ord/pull/3068) by [casey](https://github.com/casey))
- Add dry run to send, print Outgoing and PSBT ([#3063](https://github.com/ordinals/ord/pull/3063) by [raphjaph](https://github.com/raphjaph))
- Make invariant message more concise ([#3029](https://github.com/ordinals/ord/pull/3029) by [zhiqiangxu](https://github.com/zhiqiangxu))
- Forbid destinations in same-sat mode ([#3038](https://github.com/ordinals/ord/pull/3038) by [zhiqiangxu](https://github.com/zhiqiangxu))
- Exclude unnecessary docs ([#3043](https://github.com/ordinals/ord/pull/3043) by [raphjaph](https://github.com/raphjaph))
- Add documentation for reinscriptions ([#2963](https://github.com/ordinals/ord/pull/2963) by [mj10021](https://github.com/mj10021))
- Better wallet error messages ([#3041](https://github.com/ordinals/ord/pull/3041) by [raphjaph](https://github.com/raphjaph))
- Remove uneccessary allocations in Inscription Script Creation ([#3039](https://github.com/ordinals/ord/pull/3039) by [JeremyRubin](https://github.com/JeremyRubin))
- Test fee-spent inscription numbering ([#3032](https://github.com/ordinals/ord/pull/3032) by [casey](https://github.com/casey))
- Break deploy recipes into multiple lines ([#3026](https://github.com/ordinals/ord/pull/3026) by [casey](https://github.com/casey))
- Use untyped table API to get table info ([#2747](https://github.com/ordinals/ord/pull/2747) by [casey](https://github.com/casey))
- Use --name instead of --wallet in README ([#3010](https://github.com/ordinals/ord/pull/3010) by [RobertClarke](https://github.com/RobertClarke))
- Don't use browser sniffing when serving favicon ([#3003](https://github.com/ordinals/ord/pull/3003) by [casey](https://github.com/casey))
- Add minimal Dockerfile ([#2786](https://github.com/ordinals/ord/pull/2786) by [raphjaph](https://github.com/raphjaph))
- Cache less aggressively ([#3002](https://github.com/ordinals/ord/pull/3002) by [casey](https://github.com/casey))
- Remove dead link from README ([#3000](https://github.com/ordinals/ord/pull/3000) by [oxSaturn](https://github.com/oxSaturn))
- Add crate to audit content security policy ([#2993](https://github.com/ordinals/ord/pull/2993) by [casey](https://github.com/casey))
- Optimize get_inscription_ids_by_sat_paginated ([#2996](https://github.com/ordinals/ord/pull/2996) by [casey](https://github.com/casey))
- Add recipe to deploy to all servers in fleet ([#2992](https://github.com/ordinals/ord/pull/2992) by [casey](https://github.com/casey))

[0.15.0](https://github.com/ordinals/ord/releases/tag/0.15.0) - 2024-01-08
--------------------------------------------------------------------------

### Added
- Add no sync option to server command ([#2966](https://github.com/ordinals/ord/pull/2966) by [raphjaph](https://github.com/raphjaph))
- Vindicate cursed inscriptions ([#2950](https://github.com/ordinals/ord/pull/2950) by [casey](https://github.com/casey))
- Add JSON endpoints for Runes ([#2941](https://github.com/ordinals/ord/pull/2941) by [lugondev](https://github.com/lugondev))
- Add JSON endpoint for status ([#2955](https://github.com/ordinals/ord/pull/2955) by [raphjaph](https://github.com/raphjaph))
- Add chain to status page ([#2953](https://github.com/ordinals/ord/pull/2953) by [raphjaph](https://github.com/raphjaph))

### Changed
- Enter beta ([#2973](https://github.com/ordinals/ord/pull/2973) by [casey](https://github.com/casey))

### Performance
- Avoid skip when getting paginated inscriptions ([#2975](https://github.com/ordinals/ord/pull/2975) by [casey](https://github.com/casey))
- Dispatch requests to tokio thread pool ([#2974](https://github.com/ordinals/ord/pull/2974) by [casey](https://github.com/casey))

### Misc
- Fix Project Board link ([#2991](https://github.com/ordinals/ord/pull/2991) by [raphjaph](https://github.com/raphjaph))
- Update server names in justfile ([#2954](https://github.com/ordinals/ord/pull/2954) by [casey](https://github.com/casey))
- Update delegate.md ([#2976](https://github.com/ordinals/ord/pull/2976) by [gmart7t2](https://github.com/gmart7t2))
- Fix a typo ([#2980](https://github.com/ordinals/ord/pull/2980) by [GoodDaisy](https://github.com/GoodDaisy))
- Use enums for runestone tags and flags ([#2956](https://github.com/ordinals/ord/pull/2956) by [casey](https://github.com/casey))
- Make `FundRawTransactionOptions ` public ([#2938](https://github.com/ordinals/ord/pull/2938) by [lateminer](https://github.com/lateminer))
- Deduplicate deploy script case statements ([#2962](https://github.com/ordinals/ord/pull/2962) by [casey](https://github.com/casey))
- Remove quotes around key to allow shell expansion ([#2951](https://github.com/ordinals/ord/pull/2951) by [casey](https://github.com/casey))
- Restart sshd in deploy script ([#2952](https://github.com/ordinals/ord/pull/2952) by [raphjaph](https://github.com/raphjaph))

[0.14.1](https://github.com/ordinals/ord/releases/tag/0.14.1) - 2024-01-03
--------------------------------------------------------------------------

### Fixed
- Fix wallet create ([#2943](https://github.com/ordinals/ord/pull/2943) by [raphjaph](https://github.com/raphjaph))

## Misc
- Clean up justfile ([#2939](https://github.com/ordinals/ord/pull/2939) by [casey](https://github.com/casey))

[0.14.0](https://github.com/ordinals/ord/releases/tag/0.14.0) - 2024-01-02
--------------------------------------------------------------------------

### Fixed
- Keep inscriptions with unrecognized even fields unbound after jubilee ([#2894](https://github.com/ordinals/ord/pull/2894) by [casey](https://github.com/casey))

### Added
- Allow inscriptions to nominate a delegate ([#2912](https://github.com/ordinals/ord/pull/2912) by [casey](https://github.com/casey))
- Display number of times a rune has been minted ([#2901](https://github.com/ordinals/ord/pull/2901) by [casey](https://github.com/casey))
- Optionally store transactions in index ([#2885](https://github.com/ordinals/ord/pull/2885) by [casey](https://github.com/casey))
- Allow specifying destination for unallocated runes ([#2899](https://github.com/ordinals/ord/pull/2899) by [casey](https://github.com/casey))
- Make inscriptions with tag 66 permanently unbound ([#2906](https://github.com/ordinals/ord/pull/2906) by [casey](https://github.com/casey))
- Decode transactions from Bitcoin Core with `ord decode --txid` ([#2907](https://github.com/ordinals/ord/pull/2907) by [casey](https://github.com/casey))
- Allow skpping indexing inscriptions ([#2900](https://github.com/ordinals/ord/pull/2900) by [casey](https://github.com/casey))
- Add optional deadline to open etchings ([#2875](https://github.com/ordinals/ord/pull/2875) by [casey](https://github.com/casey))

### Changed
- Only store transactions with inscriptions in the database ([#2926](https://github.com/ordinals/ord/pull/2926) by [casey](https://github.com/casey))
- Hide all inscriptions with /content/<INSCRIPTION_ID> content ([#2908](https://github.com/ordinals/ord/pull/2908) by [casey](https://github.com/casey))
- Hide code, metaprotocol, and unknown media inscriptions ([#2872](https://github.com/ordinals/ord/pull/2872) by [casey](https://github.com/casey))
- Display rune symbol to right of amount ([#2871](https://github.com/ordinals/ord/pull/2871) by [casey](https://github.com/casey))

### Misc
- Use install to copy binary in deploy script ([#2934](https://github.com/ordinals/ord/pull/2934) by [casey](https://github.com/casey))
- Don't index transactions on production servers ([#2933](https://github.com/ordinals/ord/pull/2933) by [casey](https://github.com/casey))
- Add recipes to copy keys to servers ([#2927](https://github.com/ordinals/ord/pull/2927) by [casey](https://github.com/casey))
- Clean deploy/save-ord-dev-state ([#2932](https://github.com/ordinals/ord/pull/2932) by [casey](https://github.com/casey))
- Refactor bitcoin client for wallet ([#2918](https://github.com/ordinals/ord/pull/2918) by [raphjaph](https://github.com/raphjaph))
- Use enum for inscription tags ([#2921](https://github.com/ordinals/ord/pull/2921) by [casey](https://github.com/casey))
- Fix CSP origin for different deployments ([#2923](https://github.com/ordinals/ord/pull/2923) by [raphjaph](https://github.com/raphjaph))
- Placate clippy ([#2924](https://github.com/ordinals/ord/pull/2924) by [raphjaph](https://github.com/raphjaph))
- Display path to default datadir in help output ([#2881](https://github.com/ordinals/ord/pull/2881) by [torkelrogstad](https://github.com/torkelrogstad))
- Add index repair progress bar ([#2904](https://github.com/ordinals/ord/pull/2904) by [nikicat](https://github.com/nikicat))
- Listen on 127.0.0.1 to avoid firewall popup on macOS ([#2911](https://github.com/ordinals/ord/pull/2911) by [casey](https://github.com/casey))
- Set correct statistic when indexing transactions ([#2913](https://github.com/ordinals/ord/pull/2913) by [casey](https://github.com/casey))
- Show if transaction index is enabled on /status ([#2910](https://github.com/ordinals/ord/pull/2910) by [casey](https://github.com/casey))
- Optimize /inscription endpoint ([#2884](https://github.com/ordinals/ord/pull/2884) by [casey](https://github.com/casey))
- Show all inscription geneses on /tx ([#2909](https://github.com/ordinals/ord/pull/2909) by [casey](https://github.com/casey))
- Serve HTTP/2 ([#2895](https://github.com/ordinals/ord/pull/2895) by [casey](https://github.com/casey))
- Don't display trailing spacers in spaced runes ([#2896](https://github.com/ordinals/ord/pull/2896) by [casey](https://github.com/casey))
- Split runes more evenly ([#2897](https://github.com/ordinals/ord/pull/2897) by [casey](https://github.com/casey))
- Dispaly rune ID above height and index ([#2874](https://github.com/ordinals/ord/pull/2874) by [casey](https://github.com/casey))
- Use transaction version 2 ([#2873](https://github.com/ordinals/ord/pull/2873) by [casey](https://github.com/casey))

[0.13.1](https://github.com/ordinals/ord/releases/tag/0.13.1) - 2023-12-16
--------------------------------------------------------------------------

### Fixed
- Use pre-segwit transaction serialization for fundrawtransaction ([#2865](https://github.com/ordinals/ord/pull/2865) by [casey](https://github.com/casey))

[0.13.0](https://github.com/ordinals/ord/releases/tag/0.13.0) - 2023-12-15
--------------------------------------------------------------------------

### Added
- Send runes with `ord wallet send` ([#2858](https://github.com/ordinals/ord/pull/2858) by [casey](https://github.com/casey))
- Add rune spacers ([#2862](https://github.com/ordinals/ord/pull/2862) by [casey](https://github.com/casey))
- Reserve runes for sequential allocation ([#2831](https://github.com/ordinals/ord/pull/2831) by [casey](https://github.com/casey))
- Unlock runes over course of halving epoch ([#2852](https://github.com/ordinals/ord/pull/2852) by [casey](https://github.com/casey))
- Add flag to decompress brotli server-side ([#2854](https://github.com/ordinals/ord/pull/2854) by [raphjaph](https://github.com/raphjaph))
- Add /status page ([#2819](https://github.com/ordinals/ord/pull/2819) by [casey](https://github.com/casey))
- Add coin charm ([#2821](https://github.com/ordinals/ord/pull/2821) by [casey](https://github.com/casey))

### Fixed
- Fix endpoint `/inscriptions/block/<height>/<page>` ([#2798](https://github.com/ordinals/ord/pull/2798) by [gmart7t2](https://github.com/gmart7t2))

### Misc
- Tweak rune tags and flags ([#2860](https://github.com/ordinals/ord/pull/2860) by [casey](https://github.com/casey))
- Unlock runes in first block of interval ([#2861](https://github.com/ordinals/ord/pull/2861) by [casey](https://github.com/casey))
- Index runes on testnet and signet deployments ([#2857](https://github.com/ordinals/ord/pull/2857) by [casey](https://github.com/casey))
- Fix fuzzers ([#2859](https://github.com/ordinals/ord/pull/2859) by [casey](https://github.com/casey))
- Make varint decoding infallible ([#2853](https://github.com/ordinals/ord/pull/2853) by [casey](https://github.com/casey))
- Add runes to parse command ([#2830](https://github.com/ordinals/ord/pull/2830) by [casey](https://github.com/casey))
- Update dependencies ([#2828](https://github.com/ordinals/ord/pull/2828) by [casey](https://github.com/casey))
- Add coverage recipe ([#2846](https://github.com/ordinals/ord/pull/2846) by [casey](https://github.com/casey))
- Put `Accept-Encoding` value in backticks ([#2840](https://github.com/ordinals/ord/pull/2840) by [casey](https://github.com/casey))
- Don't print status when deploying ([#2838](https://github.com/ordinals/ord/pull/2838) by [casey](https://github.com/casey))
- Fix justfile ([#2836](https://github.com/ordinals/ord/pull/2836) by [raphjaph](https://github.com/raphjaph))
- Allow deploying remotes other than ordinals/ord ([#2829](https://github.com/ordinals/ord/pull/2829) by [casey](https://github.com/casey))
- Include `Accept-Encoding` header value in error message ([#2835](https://github.com/ordinals/ord/pull/2835) by [casey](https://github.com/casey))
- Clarify docs ([#2827](https://github.com/ordinals/ord/pull/2827) by [raphjaph](https://github.com/raphjaph))
- Fix batch docs ([#2823](https://github.com/ordinals/ord/pull/2823) by [raphjaph](https://github.com/raphjaph))
- Add accept encoding test without qvalues ([#2822](https://github.com/ordinals/ord/pull/2822) by [casey](https://github.com/casey))
- Italian version of the handbook ([#2801](https://github.com/ordinals/ord/pull/2801) by [DrJingLee](https://github.com/DrJingLee))
- Preview can mine blocks ([#2809](https://github.com/ordinals/ord/pull/2809) by [raphjaph](https://github.com/raphjaph))
- Burn input runes if there are no non-op-return outputs ([#2812](https://github.com/ordinals/ord/pull/2812) by [casey](https://github.com/casey))
- Update audit-cache binary ([#2804](https://github.com/ordinals/ord/pull/2804) by [casey](https://github.com/casey))

[0.12.3](https://github.com/ordinals/ord/releases/tag/0.12.3) - 2023-12-01
--------------------------------------------------------------------------

### Added
- Add `ord balances` to show rune balances ([#2782](https://github.com/ordinals/ord/pull/2782) by [casey](https://github.com/casey))

### Fixed
- Fix preview test ([#2795](https://github.com/ordinals/ord/pull/2795) by [casey](https://github.com/casey))
- Fix reinscriptions charm ([#2793](https://github.com/ordinals/ord/pull/2793) by [raphjaph](https://github.com/raphjaph))
- Fix fee calculation for batch inscribe on same sat ([#2785](https://github.com/ordinals/ord/pull/2785) by [raphjaph](https://github.com/raphjaph))

### Misc
- Add `audit-cache` binary to audit Cloudflare caching ([#2787](https://github.com/ordinals/ord/pull/2787) by [casey](https://github.com/casey))
- Fix typos ([#2791](https://github.com/ordinals/ord/pull/2791) by [vuittont60](https://github.com/vuittont60))
- Add total bytes and proportion to database info ([#2783](https://github.com/ordinals/ord/pull/2783) by [casey](https://github.com/casey))

[0.12.2](https://github.com/ordinals/ord/releases/tag/0.12.2) - 2023-11-29
--------------------------------------------------------------------------

### Added
- Bless cursed inscriptions after Jubilee height ([#2656](https://github.com/ordinals/ord/pull/2656) by [casey](https://github.com/casey))

### Misc
- Hide /content/<INSCRIPTION_ID> HTML inscriptions ([#2778](https://github.com/ordinals/ord/pull/2778) by [casey](https://github.com/casey))

[0.12.1](https://github.com/ordinals/ord/releases/tag/0.12.1) - 2023-11-29
--------------------------------------------------------------------------

### Added
- Add commands to etch and list runes ([#2544](https://github.com/ordinals/ord/pull/2544) by [casey](https://github.com/casey))
- Add ability to specify sat to batch inscribe ([#2770](https://github.com/ordinals/ord/pull/2770) by [raphjaph](https://github.com/raphjaph))
- Allow setting the sat to inscribe ([#2765](https://github.com/ordinals/ord/pull/2765) by [raphjaph](https://github.com/raphjaph))
- Batch inscribe on same sat ([#2749](https://github.com/ordinals/ord/pull/2749) by [raphjaph](https://github.com/raphjaph))
- Add stuttering curse ([#2745](https://github.com/ordinals/ord/pull/2745) by [casey](https://github.com/casey))
- Add batch to preview command ([#2752](https://github.com/ordinals/ord/pull/2752) by [raphjaph](https://github.com/raphjaph))

### Misc
- Add `public` to /content Cache-Control headers ([#2773](https://github.com/ordinals/ord/pull/2773) by [casey](https://github.com/casey))
- Set CSP origin in deploy script ([#2764](https://github.com/ordinals/ord/pull/2764) by [rot13maxi](https://github.com/rot13maxi))
- Fix typos ([#2768](https://github.com/ordinals/ord/pull/2768) by [xiaolou86](https://github.com/xiaolou86))
- Select further away coins which meet target ([#2724](https://github.com/ordinals/ord/pull/2724) by [gmart7t2](https://github.com/gmart7t2))
- Hide all text ([#2753](https://github.com/ordinals/ord/pull/2753) by [raphjaph](https://github.com/raphjaph))

[0.12.0](https://github.com/ordinals/ord/releases/tag/0.12.0) - 2023-11-24
--------------------------------------------------------------------------

### Added
- Add /r/children recursive endpoint ([#2431](https://github.com/ordinals/ord/pull/2431) by [elocremarc](https://github.com/elocremarc))
- Add sat recursive endpoints with index and pagination ([#2680](https://github.com/ordinals/ord/pull/2680) by [raphjaph](https://github.com/raphjaph))
- Allow setting CSP origin ([#2708](https://github.com/ordinals/ord/pull/2708) by [rot13maxi](https://github.com/rot13maxi))
- Add destination field to batch ([#2701](https://github.com/ordinals/ord/pull/2701) by [raphjaph](https://github.com/raphjaph))
- Preview font inscriptions ([#2692](https://github.com/ordinals/ord/pull/2692) by [elocremarc](https://github.com/elocremarc))
- Add /collections Page ([#2561](https://github.com/ordinals/ord/pull/2561) by [veryordinally](https://github.com/veryordinally))
- Add inscription compression ([#1713](https://github.com/ordinals/ord/pull/1713) by [terror](https://github.com/terror))
- Add inscription charms ([#2681](https://github.com/ordinals/ord/pull/2681) by [casey](https://github.com/casey))
- Hide protocol inscriptions ([#2674](https://github.com/ordinals/ord/pull/2674) by [casey](https://github.com/casey))
- Hide JSON and .btc ([#2744](https://github.com/ordinals/ord/pull/2744) by [raphjaph](https://github.com/raphjaph))
- Add Hindi version of handbook ([#2648](https://github.com/ordinals/ord/pull/2648) by [duttydeedz](https://github.com/duttydeedz))

### Changed
- Use icons in nav bar ([#2722](https://github.com/ordinals/ord/pull/2722) by [casey](https://github.com/casey))
- Remove default file path from `ord index export --tsv` ([#2717](https://github.com/ordinals/ord/pull/2717) by [casey](https://github.com/casey))
- Display table stats in `ord index info` ([#2711](https://github.com/ordinals/ord/pull/2711) by [casey](https://github.com/casey))
- Move postage into batch file ([#2705](https://github.com/ordinals/ord/pull/2705) by [raphjaph](https://github.com/raphjaph))

### Performance
- Use sequence numbers database keys ([#2664](https://github.com/ordinals/ord/pull/2664) by [casey](https://github.com/casey))

### Misc
- Add docs for child recursive endpoint ([#2743](https://github.com/ordinals/ord/pull/2743) by [raphjaph](https://github.com/raphjaph))
- Update docs to include all fields, including content-encoding ([#2740](https://github.com/ordinals/ord/pull/2740) by [raphjaph](https://github.com/raphjaph))
- Ignore flaky test ([#2742](https://github.com/ordinals/ord/pull/2742) by [casey](https://github.com/casey))
- Add docs and examples for sat recursive endpoint ([#2735](https://github.com/ordinals/ord/pull/2735) by [raphjaph](https://github.com/raphjaph))
- Remove `RUNE` from `<h1>` on /rune ([#2728](https://github.com/ordinals/ord/pull/2728) by [casey](https://github.com/casey))
- Add docs for metadata recursive endpoint ([#2734](https://github.com/ordinals/ord/pull/2734) by [raphjaph](https://github.com/raphjaph))
- Fix typo in docs/src/inscriptions/metadata.md ([#2731](https://github.com/ordinals/ord/pull/2731) by [vuittont60](https://github.com/vuittont60))
- Only accept sat number in recursive endpoint ([#2732](https://github.com/ordinals/ord/pull/2732) by [raphjaph](https://github.com/raphjaph))
- Add Homebrew install instructions to readme ([#2726](https://github.com/ordinals/ord/pull/2726) by [casey](https://github.com/casey))
- Add Debian packaging instructions ([#2725](https://github.com/ordinals/ord/pull/2725) by [casey](https://github.com/casey))
- Use redb's recovery callback API ([#2584](https://github.com/ordinals/ord/pull/2584) by [cberner](https://github.com/cberner))
- Refactor inscriptions paginations ([#2715](https://github.com/ordinals/ord/pull/2715) by [raphjaph](https://github.com/raphjaph))
- Update redb to 1.4.0 ([#2714](https://github.com/ordinals/ord/pull/2714) by [casey](https://github.com/casey))
- Only try to create the database if it wasn't found ([#2703](https://github.com/ordinals/ord/pull/2703) by [casey](https://github.com/casey))
- Only load used language highlight module in code preview ([#2696](https://github.com/ordinals/ord/pull/2696) by [casey](https://github.com/casey))
- Clean up install.sh ([#2669](https://github.com/ordinals/ord/pull/2669) by [eagr](https://github.com/eagr))
- Add binary media type ([#2671](https://github.com/ordinals/ord/pull/2671) by [elocremarc](https://github.com/elocremarc))
- Fix unbound outpoint server error ([#2479](https://github.com/ordinals/ord/pull/2479) by [raphjaph](https://github.com/raphjaph))
- Update schema version for charms ([#2687](https://github.com/ordinals/ord/pull/2687) by [casey](https://github.com/casey))
- Fix media table formatting ([#2686](https://github.com/ordinals/ord/pull/2686) by [casey](https://github.com/casey))
- Group rune server tests ([#2685](https://github.com/ordinals/ord/pull/2685) by [casey](https://github.com/casey))
- Don't color links in headers ([#2678](https://github.com/ordinals/ord/pull/2678) by [casey](https://github.com/casey))
- Remove Index::index_block_inscription_numbers ([#2667](https://github.com/ordinals/ord/pull/2667) by [casey](https://github.com/casey))
- Fix lost sats bug ([#2666](https://github.com/ordinals/ord/pull/2666) by [raphjaph](https://github.com/raphjaph))

[0.11.1](https://github.com/ordinals/ord/releases/tag/0.11.1) - 2023-11-09
--------------------------------------------------------------------------

### Fixed
- Use new RPC client in Reorg::get_block_with_retries ([#2650](https://github.com/ordinals/ord/pull/2650) by [casey](https://github.com/casey))

### Misc
- Refactor varint encoding ([#2645](https://github.com/ordinals/ord/pull/2645) by [eagr](https://github.com/eagr))

[0.11.0](https://github.com/ordinals/ord/releases/tag/0.11.0) - 2023-11-07
--------------------------------------------------------------------------

### Added
- Add a link to the Ordicord ([#2629](https://github.com/ordinals/ord/pull/2629) by [devords](https://github.com/devords))
- Add `/children` with pagination ([#2617](https://github.com/ordinals/ord/pull/2617) by [devords](https://github.com/devords))
- Add metadata recursive endpoint ([#2604](https://github.com/ordinals/ord/pull/2604) by [rot13maxi](https://github.com/rot13maxi))
- Add recursive directory and make all endpoints JSON ([#2493](https://github.com/ordinals/ord/pull/2493) by [raphjaph](https://github.com/raphjaph))
- Add Portuguese version of handbook ([#2572](https://github.com/ordinals/ord/pull/2572) by [namcios](https://github.com/namcios))
- Add decode just recipe ([#2592](https://github.com/ordinals/ord/pull/2592) by [casey](https://github.com/casey))
- Add `/block/:query` JSON API endpoint ([#2423](https://github.com/ordinals/ord/pull/2423) by [terror](https://github.com/terror))
- Add syntax highlighting for Python inscriptions ([#2538](https://github.com/ordinals/ord/pull/2538) by [elocremarc](https://github.com/elocremarc))
- Add publish-and-tag-crate just recipe ([#2576](https://github.com/ordinals/ord/pull/2576) by [casey](https://github.com/casey))
- Document teleburning handbook ([#2577](https://github.com/ordinals/ord/pull/2577) by [casey](https://github.com/casey))

### Changed
- Clarify sat hunting guide ([#2640](https://github.com/ordinals/ord/pull/2640) by [raphjaph](https://github.com/raphjaph))
- Update docs ([#2627](https://github.com/ordinals/ord/pull/2627) by [rot13maxi](https://github.com/rot13maxi))
- Remove blank line in CI workflow ([#2620](https://github.com/ordinals/ord/pull/2620) by [casey](https://github.com/casey))
- Update README.md and zh.po ([#2605](https://github.com/ordinals/ord/pull/2605) by [DrJingLee](https://github.com/DrJingLee))
- Require --batch or --file for `ord wallet inscribe` ([#2581](https://github.com/ordinals/ord/pull/2581) by [casey](https://github.com/casey))

### Fixed
- Respect locked coins ([#2618](https://github.com/ordinals/ord/pull/2618) by [rot13maxi](https://github.com/rot13maxi))
- Set `Cache-Control: no-store` header on 404 responses ([#2637](https://github.com/ordinals/ord/pull/2637) by [devords](https://github.com/devords))
- Fix statistics table and increment schema version ([#2624](https://github.com/ordinals/ord/pull/2624) by [raphjaph](https://github.com/raphjaph))
- Fix broken link in README ([#2621](https://github.com/ordinals/ord/pull/2621) by [yixinrock](https://github.com/yixinrock))
- Speed up indexing of re-inscriptions ([#2608](https://github.com/ordinals/ord/pull/2608) by [SmarakNayak](https://github.com/SmarakNayak))
- Fix docs rendering ([#2612](https://github.com/ordinals/ord/pull/2612) by [casey](https://github.com/casey))
- Update docs with new position of --enable-json-api ([#2601](https://github.com/ordinals/ord/pull/2601) by [elocremarc](https://github.com/elocremarc))
- Move `--enable-json-api` flag to server options ([#2599](https://github.com/ordinals/ord/pull/2599) by [raphjaph](https://github.com/raphjaph))
- Make server_runs_with_rpc_user_and_pass_as_env_vars test less flaky ([#2580](https://github.com/ordinals/ord/pull/2580) by [casey](https://github.com/casey))

### Runes
- Implement open etchings ([#2548](https://github.com/ordinals/ord/pull/2548) by [casey](https://github.com/casey))
- Add more info to /rune page and link to rune from /tx ([#2528](https://github.com/ordinals/ord/pull/2528) by [casey](https://github.com/casey))
- Display inscription on /rune ([#2542](https://github.com/ordinals/ord/pull/2542) by [casey](https://github.com/casey))
- Add rune numbers ([#2557](https://github.com/ordinals/ord/pull/2557) by [casey](https://github.com/casey))
- Ignore non push opcodes in runestones ([#2553](https://github.com/ordinals/ord/pull/2553) by [casey](https://github.com/casey))
- Improve rune minimum at height ([#2546](https://github.com/ordinals/ord/pull/2546) by [casey](https://github.com/casey))

[0.10.0](https://github.com/ordinals/ord/releases/tag/0.10.0) - 2023-10-23
--------------------------------------------------------------------------

### Added
- Batch inscriptions ([#2504](https://github.com/ordinals/ord/pull/2504) by [raphjaph](https://github.com/raphjaph))
- Add teleburn command to generate Ethereum teleburn addresses ([#1680](https://github.com/ordinals/ord/pull/1680) by [casey](https://github.com/casey))
- Add Korean version of handbook ([#2560](https://github.com/ordinals/ord/pull/2560) by [Neofishtwo](https://github.com/Neofishtwo))
- Add German version of handbook ([#2441](https://github.com/ordinals/ord/pull/2441) by [ordinaHO](https://github.com/ordinaHO))
- Add Arabic version of handbook ([#2442](https://github.com/ordinals/ord/pull/2442) by [ordinaHO](https://github.com/ordinaHO))
- Add French version of handbook ([#2508](https://github.com/ordinals/ord/pull/2508) by [rupturenft](https://github.com/rupturenft))
- Implement pointer spec ([#2499](https://github.com/ordinals/ord/pull/2499) by [raphjaph](https://github.com/raphjaph))
- Add pointer spec ([#2383](https://github.com/ordinals/ord/pull/2383) by [casey](https://github.com/casey))
- Add Russian version of handbook ([#2468](https://github.com/ordinals/ord/pull/2468) by [DrJingLee](https://github.com/DrJingLee))
- Add inscription number endpoint ([#2485](https://github.com/ordinals/ord/pull/2485) by [raphjaph](https://github.com/raphjaph))
- Allow inscriptions to include CBOR metadata ([#2421](https://github.com/ordinals/ord/pull/2421) by [casey](https://github.com/casey))
- Add Filipino version of handbook ([#2483](https://github.com/ordinals/ord/pull/2483) by [jcatama](https://github.com/jcatama))
- Add code syntax highlighting to preview ([#2471](https://github.com/ordinals/ord/pull/2471) by [elocremarc](https://github.com/elocremarc))
- Add font media types ([#2464](https://github.com/ordinals/ord/pull/2464) by [devords](https://github.com/devords))
- Render markdown previews ([#2325](https://github.com/ordinals/ord/pull/2325) by [elocremarc](https://github.com/elocremarc))
- Add metaprotocol field ([#2449](https://github.com/ordinals/ord/pull/2449) by [casey](https://github.com/casey))
- Add Spanish version of handbook ([#2448](https://github.com/ordinals/ord/pull/2448) by [Psifour](https://github.com/Psifour))
- Add `application/cbor` media type with extension `.cbor` ([#2446](https://github.com/ordinals/ord/pull/2446) by [casey](https://github.com/casey))

### Changed
- Create single-directory release archives ([#2537](https://github.com/ordinals/ord/pull/2537) by [casey](https://github.com/casey))
- Allow fixed length encoding for parent id in child inscription ([#2519](https://github.com/ordinals/ord/pull/2519) by [veryordinally](https://github.com/veryordinally))
- Recognize inscriptions with pushnum opcodes ([#2497](https://github.com/ordinals/ord/pull/2497) by [casey](https://github.com/casey))
- Rename `index run` -> `index update` ([#2462](https://github.com/ordinals/ord/pull/2462) by [casey](https://github.com/casey))
- Refactor inscription parsing ([#2461](https://github.com/ordinals/ord/pull/2461) by [casey](https://github.com/casey))
- Allow running `find` on a range of sats ([#1992](https://github.com/ordinals/ord/pull/1992) by [gmart7t2](https://github.com/gmart7t2))

### Fixed
- Fix overflow in Sat::from_name ([#2500](https://github.com/ordinals/ord/pull/2500) by [casey](https://github.com/casey))
- Fix issue with `--satpoint` when offset not 0 ([#2466](https://github.com/ordinals/ord/pull/2466) by [felipelincoln](https://github.com/felipelincoln))

### Misc
- Remove paranthetical annotations ([#2540](https://github.com/ordinals/ord/pull/2540) by [casey](https://github.com/casey))
- Refactor index checks ([#2541](https://github.com/ordinals/ord/pull/2541) by [casey](https://github.com/casey))
- Don't add path component in bin/package ([#2536](https://github.com/ordinals/ord/pull/2536) by [casey](https://github.com/casey))
- Metadata Filipino translation ([#2517](https://github.com/ordinals/ord/pull/2517) by [jcatama](https://github.com/jcatama))
- Add pointer spec to docs ([#2533](https://github.com/ordinals/ord/pull/2533) by [raphjaph](https://github.com/raphjaph))
- Make inscriptions with pointer cursed ([#2523](https://github.com/ordinals/ord/pull/2523) by [raphjaph](https://github.com/raphjaph))
- Small refactor for inscribe code ([#2515](https://github.com/ordinals/ord/pull/2515) by [raphjaph](https://github.com/raphjaph))
- Pre-allocate vector size ([#1960](https://github.com/ordinals/ord/pull/1960) by [bonedaddy](https://github.com/bonedaddy))
- Add troubleshooting guide for syncing bitcoind ([#1737](https://github.com/ordinals/ord/pull/1737) by [andrewtoth](https://github.com/andrewtoth))
- Same input envelopes become reinscriptions ([#2478](https://github.com/ordinals/ord/pull/2478) by [raphjaph](https://github.com/raphjaph))
- Document JSON-API ([#2484](https://github.com/ordinals/ord/pull/2484) by [raphjaph](https://github.com/raphjaph))
- Update parent-child guide ([#2487](https://github.com/ordinals/ord/pull/2487) by [raphjaph](https://github.com/raphjaph))
- Add regtest flag to bitcoin-cli docs ([#2488](https://github.com/ordinals/ord/pull/2488) by [elocremarc](https://github.com/elocremarc))
- Update overview.md ([#2456](https://github.com/ordinals/ord/pull/2456) by [ordinaHO](https://github.com/ordinaHO))
- Correct donation address ([#2475](https://github.com/ordinals/ord/pull/2475) by [raphjaph](https://github.com/raphjaph))
- Fixes release tarbomb ([#2473](https://github.com/ordinals/ord/pull/2473) by [uzyn](https://github.com/uzyn))
- Update dependencies ([#2470](https://github.com/ordinals/ord/pull/2470) by [casey](https://github.com/casey))
- Add internal sequence number ([#2460](https://github.com/ordinals/ord/pull/2460) by [raphjaph](https://github.com/raphjaph))
- Update guide with parent-child and json API ([#2429](https://github.com/ordinals/ord/pull/2429) by [elocremarc](https://github.com/elocremarc))
- Update Japanese handbook version with provenance section ([#2450](https://github.com/ordinals/ord/pull/2450) by [DrJingLee](https://github.com/DrJingLee))

### Runes
- Implement splits ([#2530](https://github.com/ordinals/ord/pull/2530) by [casey](https://github.com/casey))
- Add rune fuzz targets ([#2526](https://github.com/ordinals/ord/pull/2526) by [casey](https://github.com/casey))
- Allow searching by rune or rune ID ([#2522](https://github.com/ordinals/ord/pull/2522) by [casey](https://github.com/casey))
- Encode runestones with tags ([#2547](https://github.com/ordinals/ord/pull/2547) by [casey](https://github.com/casey))
- Edict with zero amount allocates all remaining runes ([#2531](https://github.com/ordinals/ord/pull/2531) by [casey](https://github.com/casey))
- Always create rune, even if none were allocated ([#2543](https://github.com/ordinals/ord/pull/2543) by [casey](https://github.com/casey))
- Show rune balances on /output page ([#2527](https://github.com/ordinals/ord/pull/2527) by [casey](https://github.com/casey))
- Delta encode Rune IDs in edicts ([#2532](https://github.com/ordinals/ord/pull/2532) by [casey](https://github.com/casey))
- Add test to keep track of runestone size ([#2529](https://github.com/ordinals/ord/pull/2529) by [casey](https://github.com/casey))
- Show etching and inscription on /rune page ([#2512](https://github.com/ordinals/ord/pull/2512) by [casey](https://github.com/casey))
- Track burned runes ([#2511](https://github.com/ordinals/ord/pull/2511) by [casey](https://github.com/casey))
- Don't encode divisibility if zero ([#2510](https://github.com/ordinals/ord/pull/2510) by [casey](https://github.com/casey))
- Format rune supply using divisibility ([#2509](https://github.com/ordinals/ord/pull/2509) by [casey](https://github.com/casey))
- Add pre-alpha unstable incomplete half-baked rune index ([#2491](https://github.com/ordinals/ord/pull/2491) by [casey](https://github.com/casey))

[0.9.0](https://github.com/ordinals/ord/releases/tag/0.9.0) - 2023-09-11
------------------------------------------------------------------------

### Added

- Allow reinscribing with wallet ([#2432](https://github.com/ordinals/ord/pull/2432) by [raphjaph](https://github.com/raphjaph))
- Provide more detailed translation instructions ([#2443](https://github.com/ordinals/ord/pull/2443) by [DrJingLee](https://github.com/DrJingLee))
- Add Japanese version of handbook ([#2426](https://github.com/ordinals/ord/pull/2426) by [DrJingLee](https://github.com/DrJingLee))
- Add provenance to docs summary ([#2427](https://github.com/ordinals/ord/pull/2427) by [casey](https://github.com/casey))
- Inscribe with parent ([#2388](https://github.com/ordinals/ord/pull/2388) by [raphjaph](https://github.com/raphjaph))
- Add provenance spec ([#2278](https://github.com/ordinals/ord/pull/2278) by [casey](https://github.com/casey))
- Implement provenance in index ([#2353](https://github.com/ordinals/ord/pull/2353) by [casey](https://github.com/casey))
- Add application/protobuf media type ([#2389](https://github.com/ordinals/ord/pull/2389) by [casey](https://github.com/casey))
- Install mdbook-i18n-helpers in Github Workflows ([#2408](https://github.com/ordinals/ord/pull/2408) by [raphjaph](https://github.com/raphjaph))
- Add `decode` command ([#2401](https://github.com/ordinals/ord/pull/2401) by [casey](https://github.com/casey))
- Add Chinese version of the handbook ([#2406](https://github.com/ordinals/ord/pull/2406) by [DrJingLee](https://github.com/DrJingLee))
- Add language picker for docs ([#2403](https://github.com/ordinals/ord/pull/2403) by [raphjaph](https://github.com/raphjaph))
- Add reindexing docs ([#2393](https://github.com/ordinals/ord/pull/2393) by [raphjaph](https://github.com/raphjaph))
- Vaccuum log with every new deploy ([#2390](https://github.com/ordinals/ord/pull/2390) by [raphjaph](https://github.com/raphjaph))

### Changed

- Fold BlockIndex into database ([#2436](https://github.com/ordinals/ord/pull/2436) by [raphjaph](https://github.com/raphjaph))
- Prevent search when query field is empty ([#2425](https://github.com/ordinals/ord/pull/2425) by [elocremarc](https://github.com/elocremarc))
- Make any zero-valued input inscription unbound ([#2397](https://github.com/ordinals/ord/pull/2397) by [raphjaph](https://github.com/raphjaph))
- Tweak translations intructions ([#2413](https://github.com/ordinals/ord/pull/2413) by [raphjaph](https://github.com/raphjaph))
- Remove unused itertools dependency ([#2416](https://github.com/ordinals/ord/pull/2416) by [casey](https://github.com/casey))
- Update dependencies ([#2414](https://github.com/ordinals/ord/pull/2414) by [casey](https://github.com/casey))
- Update clap ([#2415](https://github.com/ordinals/ord/pull/2415) by [casey](https://github.com/casey))
- Use tapscript extraction from rust-bitcoin ([#2404](https://github.com/ordinals/ord/pull/2404) by [casey](https://github.com/casey))
- Allocate blocks vector ahead of time ([#2409](https://github.com/ordinals/ord/pull/2409) by [casey](https://github.com/casey))
- Deduplicate sat range summation logic ([#2402](https://github.com/ordinals/ord/pull/2402) by [casey](https://github.com/casey))
- Inscriptions with unrecognized even fields are unbound and cursed ([#2359](https://github.com/ordinals/ord/pull/2359) by [raphjaph](https://github.com/raphjaph))
- Remove unused content_response match statement ([#2384](https://github.com/ordinals/ord/pull/2384) by [casey](https://github.com/casey))

### Fixed

- Fix type ([#2444](https://github.com/ordinals/ord/pull/2444) by [DrJingLee](https://github.com/DrJingLee))
- Fix Chinese translation typos and format errors ([#2419](https://github.com/ordinals/ord/pull/2419) by [DrJingLee](https://github.com/DrJingLee))
- Fix UTXO selection in mock Bitcoin Core instance([#2417](https://github.com/ordinals/ord/pull/2417) by [raphjaph](https://github.com/raphjaph))

[0.8.3](https://github.com/ordinals/ord/releases/tag/0.8.3) - 2023-08-28
------------------------------------------------------------------------

### Added

- Tweaks to front-end ([#2381](https://github.com/ordinals/ord/pull/2381) by [raphjaph](https://github.com/raphjaph))
- Add some links  to docs  ([#2364](https://github.com/ordinals/ord/pull/2364) by [ordinaHO](https://github.com/ordinaHO))
- Add testing guide for recursion ([#2357](https://github.com/ordinals/ord/pull/2357) by [elocremarc](https://github.com/elocremarc))
- Make homepage more interesting ([#2374](https://github.com/ordinals/ord/pull/2374) by [raphjaph](https://github.com/raphjaph))
- Add proper block inscriptions HTML ([#2337](https://github.com/ordinals/ord/pull/2337) by [veryordinally](https://github.com/veryordinally))
- Render GLB/GLTF models in preview ([#2369](https://github.com/ordinals/ord/pull/2369) by [raphjaph](https://github.com/raphjaph))
- Add tags and inscription id documentation ([#2351](https://github.com/ordinals/ord/pull/2351) by [raphjaph](https://github.com/raphjaph))
- Add hint about maximum number of open files for testing ([#2348](https://github.com/ordinals/ord/pull/2348) by [casey](https://github.com/casey))
- Reduce index durability when testing ([#2347](https://github.com/ordinals/ord/pull/2347) by [casey](https://github.com/casey))
- Homogenize design ([#2346](https://github.com/ordinals/ord/pull/2346) by [casey](https://github.com/casey))

### Fixed

- Fix slice error for inscriptions block view ([#2378](https://github.com/ordinals/ord/pull/2378) by [veryordinally](https://github.com/veryordinally))
- Use correct height and depth in reorg log ([#2352](https://github.com/ordinals/ord/pull/2352) by [gmart7t2](https://github.com/gmart7t2))

### Changed

- Remove transaction ID to inscription ID conversion ([#2370](https://github.com/ordinals/ord/pull/2370) by [casey](https://github.com/casey))
- Return JSON from all commands ([#2355](https://github.com/ordinals/ord/pull/2355) by [casey](https://github.com/casey))
- Allow splitting merged inscriptions ([#1927](https://github.com/ordinals/ord/pull/1927) by [gmart7t2](https://github.com/gmart7t2))
- Update explorer.md ([#2215](https://github.com/ordinals/ord/pull/2215) by [elocremarc](https://github.com/elocremarc))
- Recognize media types without explicit charset ([#2349](https://github.com/ordinals/ord/pull/2349) by [casey](https://github.com/casey))

[0.8.2](https://github.com/ordinals/ord/releases/tag/0.8.2) - 2023-08-17
------------------------------------------------------------------------

### Added

- Allow setting custom postage ([#2331](https://github.com/ordinals/ord/pull/2331) by [raphjaph](https://github.com/raphjaph))
- Make retrieving inscriptions in block fast ([#2333](https://github.com/ordinals/ord/pull/2333) by [veryordinally](https://github.com/veryordinally))
- JSON API for `/inscription`, `/inscriptions` and `/output` ([#2323](https://github.com/ordinals/ord/pull/2323) by [veryordinally](https://github.com/veryordinally))
- Ignore invalid content type header values ([#2326](https://github.com/ordinals/ord/pull/2326) by [casey](https://github.com/casey))
- Add reorg resistance  ([#2320](https://github.com/ordinals/ord/pull/2320) by [raphjaph](https://github.com/raphjaph))
- Add JSON API endpoint `/sat/<SAT>` ([#2250](https://github.com/ordinals/ord/pull/2250) by [Mathieu-Be](https://github.com/Mathieu-Be))
- Add `amount` field to `wallet inscriptions` output. ([#1928](https://github.com/ordinals/ord/pull/1928) by [gmart7t2](https://github.com/gmart7t2))

### Changed

- Only fetch inscriptions that are owned by the ord wallet ([#2310](https://github.com/ordinals/ord/pull/2310) by [gmart7t2](https://github.com/gmart7t2))
- Inform user when redb starts in recovery mode ([#2304](https://github.com/ordinals/ord/pull/2304) by [gmart7t2](https://github.com/gmart7t2))
- Select multiple utxos ([#2303](https://github.com/ordinals/ord/pull/2303) by [raphjaph](https://github.com/raphjaph))

### Fixed

- Use `--fee-rate` when sending an amount ([#1922](https://github.com/ordinals/ord/pull/1922) by [gmart7t2](https://github.com/gmart7t2))
- Fix typos in documentation ([#2328](https://github.com/ordinals/ord/pull/2328) by [omahs](https://github.com/omahs))
- Fix dust limit for padding in `TransactionBuilder` ([#1929](https://github.com/ordinals/ord/pull/1929) by [gmart7t2](https://github.com/gmart7t2))
- Fix remote RPC wallet commands ([#1766](https://github.com/ordinals/ord/pull/1766) by [carlosalaniz](https://github.com/carlosalaniz))

[0.8.1](https://github.com/ordinals/ord/releases/tag/0.8.1) - 2023-07-23
---------------------------------------------------------------------

### Added

- Add retry to fetcher ([#2297](https://github.com/ordinals/ord/pull/2297) by [victorkirov](https://github.com/victorkirov))
- Add satpoint and address to index export ([#2284](https://github.com/ordinals/ord/pull/2284) by [veryordinally](https://github.com/veryordinally))
- Don't create default data directory if --index overrides it ([#1991](https://github.com/ordinals/ord/pull/1991) by [gmart7t2](https://github.com/gmart7t2))
- Implement clean index shutdown to prevent index corruption (with clippy updates for Rust 1.71) ([#2275](https://github.com/ordinals/ord/pull/2275) by [raphjaph](https://github.com/raphjaph))
- Set lower max age for not found ([#2240](https://github.com/ordinals/ord/pull/2240) by [revofusion](https://github.com/revofusion))

### Changed

- Fix justfile recipe ([#2299](https://github.com/ordinals/ord/pull/2299) by [raphjaph](https://github.com/raphjaph))
- Clean up deploy scripts ([#2298](https://github.com/ordinals/ord/pull/2298) by [raphjaph](https://github.com/raphjaph))
- Update redb ([#2294](https://github.com/ordinals/ord/pull/2294) by [raphjaph](https://github.com/raphjaph))
- Update bitcoin dependencies ([#2281](https://github.com/ordinals/ord/pull/2281) by [raphjaph](https://github.com/raphjaph))
- Fix ordering for reinscriptions and show all reinscriptions for sat ([#2279](https://github.com/ordinals/ord/pull/2279) by [veryordinally](https://github.com/veryordinally))
- Modify `ord list` output to include the end of each range ([#1998](https://github.com/ordinals/ord/pull/1998) by [gmart7t2](https://github.com/gmart7t2))

### Documentation

- Fix docs inconsistency ([#2276](https://github.com/ordinals/ord/pull/2276) by [raphjaph](https://github.com/raphjaph))
- Add contributing section ([#2261](https://github.com/ordinals/ord/pull/2261) by [raphjaph](https://github.com/raphjaph))

[0.8.0](https://github.com/ordinals/ord/releases/tag/0.8.0) - 2023-07-01
---------------------------------------------------------------------

### Added

- Dev server deploy script ([#2228](https://github.com/ordinals/ord/pull/2228) by [raphjaph](https://github.com/raphjaph))
- Set DB cache size ([#2224](https://github.com/ordinals/ord/pull/2224) by [raphjaph](https://github.com/raphjaph))
- Update redb from 0.13.0 to 1.0.2 ([#2141](https://github.com/ordinals/ord/pull/2141) by [raphjaph](https://github.com/raphjaph))
- Fix typo in BIP ([#2220](https://github.com/ordinals/ord/pull/2220) by [ilanolkies](https://github.com/ilanolkies))

[0.7.0](https://github.com/ordinals/ord/releases/tag/0.7.0) - 2023-06-23
---------------------------------------------------------------------

### Added
- Tweak publish recipe ([#2212](https://github.com/ordinals/ord/pull/2212) by [raphjaph](https://github.com/raphjaph))
- Handle cursed inscriptions edge cases ([#2209](https://github.com/ordinals/ord/pull/2209) by [veryordinally](https://github.com/veryordinally))
- Add export command for <INSCRIPTION_NUMBER_TO_INSCRIPTION_ID> table ([#2208](https://github.com/ordinals/ord/pull/2208) by [raphjaph](https://github.com/raphjaph))
- Add Markdown media type ([#2206](https://github.com/ordinals/ord/pull/2206) by [elocremarc](https://github.com/elocremarc))
- Add blob urls to Content Security Policy headers ([#2203](https://github.com/ordinals/ord/pull/2203) by [Vanniix](https://github.com/Vanniix))
- Check inscribe destination address network ([#2189](https://github.com/ordinals/ord/pull/2189) by [casey](https://github.com/casey))

[0.6.2](https://github.com/ordinals/ord/releases/tag/0.6.2) - 2023-06-15
---------------------------------------------------------------------

### Added
- Recursive endpoints: `/blockhash, /blockheight, /blocktime` ([#2175](https://github.com/ordinals/ord/pull/2175) by [raphjaph](https://github.com/raphjaph))
- Document recursion ([#2174](https://github.com/ordinals/ord/pull/2174) by [casey](https://github.com/casey))
- Add CSS and JavaScript media types ([#2173](https://github.com/ordinals/ord/pull/2173) by [casey](https://github.com/casey))
- Recursive Inscriptions ([#2167](https://github.com/ordinals/ord/pull/2167) by [casey](https://github.com/casey))

### Misc
- Update ord dependency in lockfile ([#2168](https://github.com/ordinals/ord/pull/2168) by [casey](https://github.com/casey))

[0.6.1](https://github.com/ordinals/ord/releases/tag/0.6.1) - 2023-06-06
---------------------------------------------------------------------

### Changed
- Fix sat index test and unbound assignment ([#2154](https://github.com/ordinals/ord/pull/2154) by [raphjaph](https://github.com/raphjaph))
- Updated install.sh for new repo name ([#2155](https://github.com/ordinals/ord/pull/2155) by [LightRider5](https://github.com/LightRider5))

[0.6.0](https://github.com/ordinals/ord/releases/tag/0.6.0) - 2023-06-04
---------------------------------------------------------------------

### Added
- Cursed Inscriptions [1/n] ([#2145](https://github.com/ordinals/ord/pull/2145) by [raphjaph](https://github.com/raphjaph))
- Authenticate to bitcoin using a username and password ([#1527](https://github.com/ordinals/ord/pull/1527) by [raphjaph](https://github.com/raphjaph))
- Add example config file ([#2044](https://github.com/ordinals/ord/pull/2044) by [soenkehahn](https://github.com/soenkehahn))

### Changed
- Unbind inscriptions from zero-sat transactions ([#2107](https://github.com/ordinals/ord/pull/2107) by [casey](https://github.com/casey))

### Documentation
- Tweak doc: Inscriptions made on first sat of input ([#2148](https://github.com/ordinals/ord/pull/2148) by [raphjaph](https://github.com/raphjaph))
- `OP_PUSH` instead of `OP_1` in inscription docs ([#2135](https://github.com/ordinals/ord/pull/2135) by [raphjaph](https://github.com/raphjaph))
- Document bitcoind RPC authentication options ([#2056](https://github.com/ordinals/ord/pull/2056) by [casey](https://github.com/casey))
- Fix typo in Sparrow Wallet docs ([#2077](https://github.com/ordinals/ord/pull/2077) by [eltociear](https://github.com/eltociear))
- Update donate.md for inscriptions donations. ([#2125](https://github.com/ordinals/ord/pull/2125) by [veryordinally](https://github.com/veryordinally))
- Promote raphjaph to lead maintainer 🫡 ([#2119](https://github.com/ordinals/ord/pull/2119) by [casey](https://github.com/casey))
- Improve donation page ([#2034](https://github.com/ordinals/ord/pull/2034) by [casey](https://github.com/casey))

### Misc
- Switch CI back to stable clippy ([#2108](https://github.com/ordinals/ord/pull/2108) by [casey](https://github.com/casey))
- Update dependencies ([#2068](https://github.com/ordinals/ord/pull/2068) by [casey](https://github.com/casey))
- Use struct variants in Origin enum ([#2067](https://github.com/ordinals/ord/pull/2067) by [casey](https://github.com/casey))
- Fix test name typos([#2043](https://github.com/ordinals/ord/pull/2043) by [soenkehahn](https://github.com/soenkehahn))
- Switch to nightly clippy ([#2037](https://github.com/ordinals/ord/pull/2037) by [soenkehahn](https://github.com/soenkehahn))

[0.5.2](https://github.com/ordinals/ord/releases/tag/0.5.2) - 2023-04-17
---------------------------------------------------------------------

### Added
- Add `ord wallet cardinals` command to list the cardinal outputs ([#1904](https://github.com/ordinals/ord/pull/1904) by [gmart7t2](https://github.com/gmart7t2))

### Changed
- Shut down immediately after two interrupts ([#2008](https://github.com/ordinals/ord/pull/2008) by [terror](https://github.com/terror))
- Mandatory fee rate for inscribe ([#1897](https://github.com/ordinals/ord/pull/1897) by [gmart7t2](https://github.com/gmart7t2))
- Add error when a satpoint's offset exceeds the size of its output ([#1857](https://github.com/ordinals/ord/pull/1857) by [gmart7t2](https://github.com/gmart7t2))

### Fixed
- Fix fee-spent inscription tracking ([#1971](https://github.com/ordinals/ord/pull/1971) by [gmart7t2](https://github.com/gmart7t2))
- Label change and receive addresses correctly ([#1847](https://github.com/ordinals/ord/pull/1847) by [gmart7t2](https://github.com/gmart7t2))
- Correct reveal tx fee calculation ([#1853](https://github.com/ordinals/ord/pull/1853) by [gmart7t2](https://github.com/gmart7t2))

### Misc
- Misc changes ([#2025](https://github.com/ordinals/ord/pull/2025) by [casey](https://github.com/casey))
- Misc doc fixes ([#2021](https://github.com/ordinals/ord/pull/2021) by [raphjaph](https://github.com/raphjaph))
- Typo in sparrow wallet guide ([#1947](https://github.com/ordinals/ord/pull/1947) by [gmart7t2](https://github.com/gmart7t2))
- Miscellaneous design improvements ([#1968](https://github.com/ordinals/ord/pull/1968) by [raphjaph](https://github.com/raphjaph))
- Update miniscript dependency to 9.0.1 ([#1966](https://github.com/ordinals/ord/pull/1966) by [soenkehahn](https://github.com/soenkehahn))
- Skip indexing inscriptions when below first inscription also for `--index-sats`([#1828](https://github.com/ordinals/ord/pull/1828) by [andrewtoth](https://github.com/andrewtoth))
- Better interrupt message ([#1874](https://github.com/ordinals/ord/pull/1874) by [neunenak](https://github.com/neunenak))
- Fix colored coins link in BIP ([#1856](https://github.com/ordinals/ord/pull/1856) by [gmart7t2](https://github.com/gmart7t2))
- Added cozy pair programming twitch link to README.md ([#1827](https://github.com/ordinals/ord/pull/1827) by [cbspears](https://github.com/cbspears))
- Create rpc client after updating index ([#1731](https://github.com/ordinals/ord/pull/1731) by [andrewtoth](https://github.com/andrewtoth))
- Add additional err msg to build from source for users who's arch falls outside of the list ([#1792](https://github.com/ordinals/ord/pull/1792) by [bnonni](https://github.com/bnonni))
- Add note on default build location ([#1625](https://github.com/ordinals/ord/pull/1625) by [whoabuddy](https://github.com/whoabuddy))
- Minor copy fixes ([#1730](https://github.com/ordinals/ord/pull/1730) by [kn0wmad](https://github.com/kn0wmad))
- Typo ([#1815](https://github.com/ordinals/ord/pull/1815) by [toddynho](https://github.com/toddynho))

[0.5.1](https://github.com/ordinals/ord/releases/tag/0.5.1) - 2023-02-21
---------------------------------------------------------------------

### Performance
- Batch tx requests and re-enable skipping transactions ([#1759](https://github.com/ordinals/ord/pull/1759) by [andrewtoth](https://github.com/andrewtoth))

### Added
- Add option to set inscription destination address ([#1536](https://github.com/ordinals/ord/pull/1536) by [rot13maxi](https://github.com/rot13maxi))
- Allow supplying passphrase for `ord wallet create` and `ord wallet restore` ([#1669](https://github.com/ordinals/ord/pull/1669) by [Psifour](https://github.com/Psifour))
- Add `--config-dir` option ([#1697](https://github.com/ordinals/ord/pull/1697) by [terror](https://github.com/terror))

### Changed
- Require users manually specify a `--fee-rate` for `wallet send` ([#1755](https://github.com/ordinals/ord/pull/1755) by [veryordinally](https://github.com/veryordinally))

### Documentation
- Add Sparrow Wallet Guide to Handbook ([#1742](https://github.com/ordinals/ord/pull/1742) by [windsok](https://github.com/windsok))

### Misc
- Handle block count RPC error gracefully ([#1637](https://github.com/ordinals/ord/pull/1637) by [andrewtoth](https://github.com/andrewtoth))
- Fix typos in overview.md ([#1715](https://github.com/ordinals/ord/pull/1715) by [mjethani](https://github.com/mjethani))
- Typo fix ([#1682](https://github.com/ordinals/ord/pull/1682) by [sbddesign](https://github.com/sbddesign))
- README typo fix ([#1716](https://github.com/ordinals/ord/pull/1716) by [terror](https://github.com/terror))
- Fix changelog dates: 2022 → 2023 ([#1700](https://github.com/ordinals/ord/pull/1700) by [casey](https://github.com/casey))
- Bump version number ([#1695](https://github.com/ordinals/ord/pull/1695) by [casey](https://github.com/casey))

[0.5.0](https://github.com/ordinals/ord/releases/tag/0.5.0) - 2023-02-11
---------------------------------------------------------------------

### Breaking Changes
- Upgrade to redb 0.13.0 ([#1513](https://github.com/ordinals/ord/pull/1513) by [hantuzun](https://github.com/hantuzun))
- Update redb to 0.12.1 ([#1329](https://github.com/ordinals/ord/pull/1329) by [casey](https://github.com/casey))
- Display inscription genesis fee ([#1381](https://github.com/ordinals/ord/pull/1381) by [raphjaph](https://github.com/raphjaph))

### Added
- Add support for `.glb` inscriptions ([#1689](https://github.com/ordinals/ord/pull/1689) by [casey](https://github.com/casey))
- Add --no-limit flag to bypass MAX_STANDARD_TX_WEIGHT check to allow four meggers ([#1571](https://github.com/ordinals/ord/pull/1571) by [raphjaph](https://github.com/raphjaph))
- Add `--commit-fee-rate` for setting inscribe commit transaction fee rate ([#1490](https://github.com/ordinals/ord/pull/1490) by [andrewtoth](https://github.com/andrewtoth))
- Allow viewing but not creating AVIF inscriptions ([#1428](https://github.com/ordinals/ord/pull/1428) by [hashbender](https://github.com/hashbender))
- Support STL inscriptions ([#1492](https://github.com/ordinals/ord/pull/1492) by [casey](https://github.com/casey))
- Support MP4 inscriptions ([#1419](https://github.com/ordinals/ord/pull/1419) by [cryptoquick](https://github.com/cryptoquick))
- Preview JSON and YAML inscriptions as text ([#1449](https://github.com/ordinals/ord/pull/1449) by [casey](https://github.com/casey))
- Display inputs on /tx ([#1433](https://github.com/ordinals/ord/pull/1433) by [casey](https://github.com/casey))
- Support PGP signature inscriptions ([#1413](https://github.com/ordinals/ord/pull/1413) by [casey](https://github.com/casey))
- Add config ([#1392](https://github.com/ordinals/ord/pull/1392) by [casey](https://github.com/casey))
- Add paging to /inscriptions ([#1279](https://github.com/ordinals/ord/pull/1279) by [casey](https://github.com/casey))

### Changed
- Increase deployment mempool size to 1024 megabytes ([#1587](https://github.com/ordinals/ord/pull/1587) by [casey](https://github.com/casey))
- Increase number of inscriptions in RSS feed ([#1567](https://github.com/ordinals/ord/pull/1567) by [raphjaph](https://github.com/raphjaph))
- Link to block from /inscription ([#1395](https://github.com/ordinals/ord/pull/1395) by [casey](https://github.com/casey))
- Use favicon as icon for Twitter preview ([#1425](https://github.com/ordinals/ord/pull/1425) by [raphjaph](https://github.com/raphjaph))
- Allow data URIs in content security policy ([#1422](https://github.com/ordinals/ord/pull/1422) by [casey](https://github.com/casey))
- Raise server open file limit ([#1408](https://github.com/ordinals/ord/pull/1408) by [casey](https://github.com/casey))
- Remove HTTP to HTTPS redirect ([#1414](https://github.com/ordinals/ord/pull/1414) by [casey](https://github.com/casey))
- Use JSON for more command output ([#1367](https://github.com/ordinals/ord/pull/1367) by [raphjaph](https://github.com/raphjaph))
- Use JSON for `wallet` command output ([#1359](https://github.com/ordinals/ord/pull/1359) by [raphjaph](https://github.com/raphjaph))

### Misc
- Set rustc version in Cargo.toml & README ([#1615](https://github.com/ordinals/ord/pull/1615) by [apbendi](https://github.com/apbendi))
- Disable Prettier format-on-save ([#1593](https://github.com/ordinals/ord/pull/1593) by [whoabuddy](https://github.com/whoabuddy))
- Add build instructions to README ([#1573](https://github.com/ordinals/ord/pull/1573) by [dplusplus1024](https://github.com/dplusplus1024))
- Ensure wallet commands load wallet ([#1524](https://github.com/ordinals/ord/pull/1524) by [raphjaph](https://github.com/raphjaph))
- Improve error messages related to cookie file ([#1537](https://github.com/ordinals/ord/pull/1537) by [casey](https://github.com/casey))
- Include inscription ID in text inscription decode error ([#1540](https://github.com/ordinals/ord/pull/1540) by [casey](https://github.com/casey))
- Lazily load iframes ([#1456](https://github.com/ordinals/ord/pull/1456) by [casey](https://github.com/casey))
- Log recoverable errors as warnings
- Add alert pop-up example ([#1498](https://github.com/ordinals/ord/pull/1498) by [casey](https://github.com/casey))
- Use custom Discord invite link in handbox ([#1506](https://github.com/ordinals/ord/pull/1506) by [casey](https://github.com/casey))
- Note that bounty 3 requires sat index ([#1509](https://github.com/ordinals/ord/pull/1509) by [casey](https://github.com/casey))
- Link donation addresses to mempool.space ([#1510](https://github.com/ordinals/ord/pull/1510) by [casey](https://github.com/casey))
- Add linebreak to donate page ([#1500](https://github.com/ordinals/ord/pull/1500) by [casey](https://github.com/casey))
- Add donate page to handbook ([#1499](https://github.com/ordinals/ord/pull/1499) by [casey](https://github.com/casey))
- Moderation guide typo: wiht → with ([#1483](https://github.com/ordinals/ord/pull/1483) by [casey](https://github.com/casey))
- Add moderation guide ([#1473](https://github.com/ordinals/ord/pull/1473) by [casey](https://github.com/casey))
- Add collecting guide to docs ([#1474](https://github.com/ordinals/ord/pull/1474) by [casey](https://github.com/casey))
- Add missing dependencies to shell.nix ([#1463](https://github.com/ordinals/ord/pull/1463) by [niftynei](https://github.com/niftynei))
- Mute and autoplay video inscriptions ([#1420](https://github.com/ordinals/ord/pull/1420) by [cryptoquick](https://github.com/cryptoquick))
- Throw an error Bitcoin Core wallet and ord index are out of sync ([#1459](https://github.com/ordinals/ord/pull/1459) by [raphjaph](https://github.com/raphjaph))
- Typo: managment -> management ([#1441](https://github.com/ordinals/ord/pull/1441) by [jsahagun91](https://github.com/jsahagun91))
- Fix README.md grammar ([#1406](https://github.com/ordinals/ord/pull/1406) by [rayonx](https://github.com/rayonx))
- Typo: Aritacts -> Artifacts ([#1434](https://github.com/ordinals/ord/pull/1434) by [worm-emoji](https://github.com/worm-emoji))
- Update justfile to use unproxied domains ([#1391](https://github.com/ordinals/ord/pull/1391) by [casey](https://github.com/casey))
- Typo: sat -> sats ([#1411](https://github.com/ordinals/ord/pull/1411) by [goodwinmark](https://github.com/goodwinmark))
- Docs: `ord wallet utxos` -> `ord wallet outputs` ([#1405](https://github.com/ordinals/ord/pull/1405) by [cryptoquick](https://github.com/cryptoquick))
- Round expected sat timestamps ([#1386](https://github.com/ordinals/ord/pull/1386) by [casey](https://github.com/casey))
- Remove ellipsis ([#1376](https://github.com/ordinals/ord/pull/1376) by [casey](https://github.com/casey))
- Hide overflowing ordered lists ([#1384](https://github.com/ordinals/ord/pull/1384) by [casey](https://github.com/casey))
- Compress responses ([#1366](https://github.com/ordinals/ord/pull/1366) by [casey](https://github.com/casey))
- Avoid listening on 0.0.0.0 in tests ([#1365](https://github.com/ordinals/ord/pull/1365) by [raphjaph](https://github.com/raphjaph))
- Rename `GitHub` nav link to `Wallet` ([#1360](https://github.com/ordinals/ord/pull/1360) by [casey](https://github.com/casey))

[0.4.2](https://github.com/ordinals/ord/releases/tag/0.4.2) - 2023-01-24
---------------------------------------------------------------------

### Changed
- Fetch transactions below first inscription height

### Fixed
- Fix install script directory ([#1356](https://github.com/ordinals/ord/pull/1356) by [casey](https://github.com/casey))

### Misc
- Fix guide typo: getblockchount -> getblockcount ([#1354](https://github.com/ordinals/ord/pull/1354) by [casey](https://github.com/casey))

[0.4.1](https://github.com/ordinals/ord/releases/tag/0.4.1) - 2023-01-24
---------------------------------------------------------------------

### Added
- Support video inscriptions ([#1349](https://github.com/ordinals/ord/pull/1349) by [casey](https://github.com/casey))
- Support PDF Inscriptions ([#1352](https://github.com/ordinals/ord/pull/1352) by [casey](https://github.com/casey))
- Display lost sats on /output ([#1336](https://github.com/ordinals/ord/pull/1336) by [casey](https://github.com/casey))
- Show explorer URLs in `ord wallet inscriptions` ([#1308](https://github.com/ordinals/ord/pull/1308) by [raphjaph](https://github.com/raphjaph))

### Changed
- Display timestamps as UTC ([#1348](https://github.com/ordinals/ord/pull/1348) by [casey](https://github.com/casey))
- Enable pointer events on inscription page iframes ([#1344](https://github.com/ordinals/ord/pull/1344) by [casey](https://github.com/casey))
- Exclude inscribed utxos when calculating wallet balance ([#1341](https://github.com/ordinals/ord/pull/1341) by [raphjaph](https://github.com/raphjaph))

### Misc
- Activate nav arrows on single tap on iOS Safari ([#1347](https://github.com/ordinals/ord/pull/1347) by [casey](https://github.com/casey))
- Ignore keyboard events search box has focus ([#1346](https://github.com/ordinals/ord/pull/1346) by [casey](https://github.com/casey))
- Cache content responses ([#1318](https://github.com/ordinals/ord/pull/1318) by [casey](https://github.com/casey))
- Show unordered list decorations ([#1353](https://github.com/ordinals/ord/pull/1353) by [casey](https://github.com/casey))
- Update inscriptions guide for mainnet ([#1342](https://github.com/ordinals/ord/pull/1342) by [casey](https://github.com/casey))
- Hide list overflow and break dl overflow between words ([#1343](https://github.com/ordinals/ord/pull/1343) by [raphjaph](https://github.com/raphjaph))
- Add white on black fish eye logo ([#1325](https://github.com/ordinals/ord/pull/1325) by [casey](https://github.com/casey))
- Un-reverse thumbnail row order ([#1321](https://github.com/ordinals/ord/pull/1321) by [casey](https://github.com/casey))
- Deploy branches other than master to mainnet ([#1319](https://github.com/ordinals/ord/pull/1319) by [casey](https://github.com/casey))
- Add Just recipe to deploy to all chains ([#1313](https://github.com/ordinals/ord/pull/1313) by [casey](https://github.com/casey))
- Display newest inscriptions on right ([#1311](https://github.com/ordinals/ord/pull/1311) by [casey](https://github.com/casey))
- Allow publishing arbitrary revisions with publish recipe ([#1307](https://github.com/ordinals/ord/pull/1307) by [casey](https://github.com/casey))
- Make genesis clock mark orange and add tooltip to height ([#1312](https://github.com/ordinals/ord/pull/1312) by [casey](https://github.com/casey))
- Serve favicon as PNG to Safari and SVG others ([#1302](https://github.com/ordinals/ord/pull/1302) by [casey](https://github.com/casey))
- Use sans-serif font for height on clock ([#1300](https://github.com/ordinals/ord/pull/1300) by [casey](https://github.com/casey))

[0.4.0](https://github.com/ordinals/ord/releases/tag/0.4.0) - 2023-01-19
---------------------------------------------------------------------

### Added
- Support searching for inscription IDs ([#1294](https://github.com/ordinals/ord/pull/1294) by [raphjaph](https://github.com/raphjaph))
- Add RSS feed ([#1229](https://github.com/ordinals/ord/pull/1229) by [casey](https://github.com/casey))
- Add --dry-run-flag ([#1265](https://github.com/ordinals/ord/pull/1265) by [raphjaph](https://github.com/raphjaph))
- Support recovering wallet from mnemonic ([#1215](https://github.com/ordinals/ord/pull/1215) by [casey](https://github.com/casey))
- Audio inscriptions ([#1241](https://github.com/ordinals/ord/pull/1241) by [casey](https://github.com/casey))
- Allow using custom fee rate ([#1150](https://github.com/ordinals/ord/pull/1150) by [raphjaph](https://github.com/raphjaph))
- Show timestamp on /inscription ([#1200](https://github.com/ordinals/ord/pull/1200) by [casey](https://github.com/casey))
- Add prev and next links to /inscription ([#1193](https://github.com/ordinals/ord/pull/1193) by [casey](https://github.com/casey))
- Show address on /inscription ([#1187](https://github.com/ordinals/ord/pull/1187) by [casey](https://github.com/casey))
- Add --limit to `ord wallet transaction` ([#1049](https://github.com/ordinals/ord/pull/1049) by [rot13maxi](https://github.com/rot13maxi))
- Add `ord preview` ([#1089](https://github.com/ordinals/ord/pull/1089) by [casey](https://github.com/casey))
- Add `ord wallet balance` ([#1047](https://github.com/ordinals/ord/pull/1047) by [rot13maxi](https://github.com/rot13maxi))
- Support HTML and SVG inscriptions ([#1035](https://github.com/ordinals/ord/pull/1035) by [casey](https://github.com/casey))
- Display genesis height on inscription page ([#1026](https://github.com/ordinals/ord/pull/1026) by [raphjaph](https://github.com/raphjaph))
- Support more image types ([#1020](https://github.com/ordinals/ord/pull/1020) by [casey](https://github.com/casey))
- Support GIFs ([#1013](https://github.com/ordinals/ord/pull/1013) by [raphjaph](https://github.com/raphjaph))

### Changed
- Poll Bitcoin Core less frequently ([#1268](https://github.com/ordinals/ord/pull/1268) by [raphjaph](https://github.com/raphjaph))
- Automatically load wallet ([#1210](https://github.com/ordinals/ord/pull/1210) by [raphjaph](https://github.com/raphjaph))
- Ignore inscriptions on sat after first ([#1214](https://github.com/ordinals/ord/pull/1214) by [casey](https://github.com/casey))
- Allow right-click to save image inscriptions ([#1218](https://github.com/ordinals/ord/pull/1218) by [casey](https://github.com/casey))
- Scale text inscriptions to fit preview ([#1222](https://github.com/ordinals/ord/pull/1222) by [casey](https://github.com/casey))
- Convert `ord wallet inscriptions` to JSON ([#1224](https://github.com/ordinals/ord/pull/1224) by [casey](https://github.com/casey))
- Improve error when preview fails to launch bitcoind ([#1243](https://github.com/ordinals/ord/pull/1243) by [casey](https://github.com/casey))
- Output inscription ID from `ord wallet inscribe` ([#1208](https://github.com/ordinals/ord/pull/1208) by [casey](https://github.com/casey))
- Allow arbitrary wallet names ([#1207](https://github.com/ordinals/ord/pull/1207) by [raphjaph](https://github.com/raphjaph))
- Use distinct inscription IDs ([#1201](https://github.com/ordinals/ord/pull/1201) by [casey](https://github.com/casey))
- Remove ordinal addresses ([#1197](https://github.com/ordinals/ord/pull/1197) by [casey](https://github.com/casey))
- Create taproot-only wallets ([#1158](https://github.com/ordinals/ord/pull/1158) by [raphjaph](https://github.com/raphjaph))
- Check schema when opening index ([#1127](https://github.com/ordinals/ord/pull/1127) by [casey](https://github.com/casey))
- Teach `ord wallet send` to send cardinal sats ([#1137](https://github.com/ordinals/ord/pull/1137) by [casey](https://github.com/casey))
- Rename `ord wallet utxos` → `ord wallet outputs` ([#1148](https://github.com/ordinals/ord/pull/1148) by [casey](https://github.com/casey))
- Swap arguments to ord wallet send ([#1142](https://github.com/ordinals/ord/pull/1142) by [casey](https://github.com/casey))
- Rename --index-satoshis → --index-sats ([#993](https://github.com/ordinals/ord/pull/993) by [casey](https://github.com/casey))

### Fixed
- Fix preview for inscriptions with no body ([#1287](https://github.com/ordinals/ord/pull/1287) by [casey](https://github.com/casey))
- Bail if reveal transaction is too large ([#1272](https://github.com/ordinals/ord/pull/1272) by [casey](https://github.com/casey))
- Increase commit transaction output to pay for reveal transaction ([#1242](https://github.com/ordinals/ord/pull/1242) by [casey](https://github.com/casey))
- Fix inscription thumbnail links ([#1199](https://github.com/ordinals/ord/pull/1199) by [casey](https://github.com/casey))
- Use outpoint value table correctly and cache values in memory([#1172](https://github.com/ordinals/ord/pull/1172) by [casey](https://github.com/casey))
- Fix install script targets ([#1120](https://github.com/ordinals/ord/pull/1120) by [casey](https://github.com/casey))

### Misc
- Use examples in core preview test ([#1289](https://github.com/ordinals/ord/pull/1289) by [raphjaph](https://github.com/raphjaph))
- Use array for transaction builder change addresses ([#1281](https://github.com/ordinals/ord/pull/1281) by [casey](https://github.com/casey))
- Fuzz test TransactionBuilder ([#1283](https://github.com/ordinals/ord/pull/1283) by [raphjaph](https://github.com/raphjaph))
- Adopt Fish Eye logo ([#1270](https://github.com/ordinals/ord/pull/1270) by [casey](https://github.com/casey))
- Split library and binary ([#1273](https://github.com/ordinals/ord/pull/1273) by [casey](https://github.com/casey))
- Fix preview kill on drop ([#1260](https://github.com/ordinals/ord/pull/1260) by [raphjaph](https://github.com/raphjaph))
- Add warning to readme ([#1213](https://github.com/ordinals/ord/pull/1213) by [casey](https://github.com/casey))
- Run ignored tests in `ci` recipe ([#1259](https://github.com/ordinals/ord/pull/1259) by [casey](https://github.com/casey))
- Add Bitcoin Core test job to CI ([#1191](https://github.com/ordinals/ord/pull/1191) by [casey](https://github.com/casey))
- Add digital artifacts page to handbook ([#1165](https://github.com/ordinals/ord/pull/1165) by [casey](https://github.com/casey))
- Use numbers in page titles ([#1221](https://github.com/ordinals/ord/pull/1221) by [casey](https://github.com/casey))
- Set strict transport security header ([#1216](https://github.com/ordinals/ord/pull/1216) by [casey](https://github.com/casey))
- Simplify BIP ([#1226](https://github.com/ordinals/ord/pull/1226) by [casey](https://github.com/casey))
- Document required Bitcoin Core version for inscribing ([#1225](https://github.com/ordinals/ord/pull/1225) by [casey](https://github.com/casey))
- Index lost sat ranges ([#1227](https://github.com/ordinals/ord/pull/1227) by [casey](https://github.com/casey))
- Link to /block from /sat ([#1228](https://github.com/ordinals/ord/pull/1228) by [casey](https://github.com/casey))
- Print index path in `ord info` ([#1232](https://github.com/ordinals/ord/pull/1232) by [casey](https://github.com/casey))
- Add backlinks from /output and /transaction ([#1235](https://github.com/ordinals/ord/pull/1235) by [casey](https://github.com/casey))
- Don't check lockfile on CI ([#1209](https://github.com/ordinals/ord/pull/1209) by [casey](https://github.com/casey))
- Redirect HTTP to HTTPS ([#1198](https://github.com/ordinals/ord/pull/1198) by [casey](https://github.com/casey))
- Test that inscriptions in additional envelopes and outputs are ignored ([#1190](https://github.com/ordinals/ord/pull/1190) by [casey](https://github.com/casey))
- Use "sat" throughout codebase ([#1184](https://github.com/ordinals/ord/pull/1184) by [casey](https://github.com/casey))
- Enable firewall on deployments ([#1186](https://github.com/ordinals/ord/pull/1186) by [casey](https://github.com/casey))
- Request bech32m addresses in preview command ([#1183](https://github.com/ordinals/ord/pull/1183) by [casey](https://github.com/casey))
- Use mainnet in tests ([#1185](https://github.com/ordinals/ord/pull/1185) by [casey](https://github.com/casey))
- Move wallet tests into submodules ([#1182](https://github.com/ordinals/ord/pull/1182) by [casey](https://github.com/casey))
- Link to /sat from /inscription ([#1176](https://github.com/ordinals/ord/pull/1176) by [casey](https://github.com/casey))
- Match inscription preview and site background colors ([#1175](https://github.com/ordinals/ord/pull/1175) by [casey](https://github.com/casey))
- Test that envelopes not starting with OP_FALSE are ignored ([#1171](https://github.com/ordinals/ord/pull/1171) by [casey](https://github.com/casey))
- Update changelog ([#1177](https://github.com/ordinals/ord/pull/1177) by [casey](https://github.com/casey))
- Remove mainnet wall restrictions ([#1170](https://github.com/ordinals/ord/pull/1170) by [casey](https://github.com/casey))
- Ordinal addresses ([#1163](https://github.com/ordinals/ord/pull/1163) by [casey](https://github.com/casey))
- Link outputs and inscriptions ([#1162](https://github.com/ordinals/ord/pull/1162) by [casey](https://github.com/casey))
- Remove mainnet ord-dev index ([#1164](https://github.com/ordinals/ord/pull/1164) by [casey](https://github.com/casey))
- Preview all inscriptions inside iframes ([#1132](https://github.com/ordinals/ord/pull/1132) by [casey](https://github.com/casey))
- Remove inscriptions subcommand struct ([#1151](https://github.com/ordinals/ord/pull/1151) by [casey](https://github.com/casey))
- Limit transaction count limit to u16::Max ([#1152](https://github.com/ordinals/ord/pull/1152) by [casey](https://github.com/casey))
- Tweak homepage ([#1124](https://github.com/ordinals/ord/pull/1124) by [casey](https://github.com/casey))
- Track fee-spent and lost inscriptions ([#1125](https://github.com/ordinals/ord/pull/1125) by [casey](https://github.com/casey))
- Use InscriptionId in Reference ([#1135](https://github.com/ordinals/ord/pull/1135) by [casey](https://github.com/casey))
- Avoid push_scriptint ([#1136](https://github.com/ordinals/ord/pull/1136) by [casey](https://github.com/casey))
- Check Bitcoin Core version before inscribing ([#1048](https://github.com/ordinals/ord/pull/1048) by [rot13maxi](https://github.com/rot13maxi))
- Display alpha in navbar on mainnet ([#1122](https://github.com/ordinals/ord/pull/1122) by [casey](https://github.com/casey))
- Make PageHtml generic over PageContent type ([#1123](https://github.com/ordinals/ord/pull/1123) by [casey](https://github.com/casey))
- Track inscriptions at offset and vout other than first ([#1108](https://github.com/ordinals/ord/pull/1108) by [casey](https://github.com/casey))
- Unrecognized even fields are invalid ([#1107](https://github.com/ordinals/ord/pull/1107) by [casey](https://github.com/casey))
- Add short flags ([#1102](https://github.com/ordinals/ord/pull/1102) by [casey](https://github.com/casey))
- Document Debian dependencies ([#1110](https://github.com/ordinals/ord/pull/1110) by [casey](https://github.com/casey))
- Add first testnet inscription height ([#1109](https://github.com/ordinals/ord/pull/1109) by [casey](https://github.com/casey))
- Remove CORS headers ([#1103](https://github.com/ordinals/ord/pull/1103) by [casey](https://github.com/casey))
- Don't wrap text inscriptions ([#1100](https://github.com/ordinals/ord/pull/1100) by [casey](https://github.com/casey))
- Upgrade to redb 0.11.0 ([#1099](https://github.com/ordinals/ord/pull/1099) by [cberner](https://github.com/cberner))
- Add quickstart script for macos ([#1096](https://github.com/ordinals/ord/pull/1096) by [casey](https://github.com/casey))
- Remove text inscription anchor tag text decoration ([#1084](https://github.com/ordinals/ord/pull/1084) by [casey](https://github.com/casey))
- Display inscription content on /inscriptions ([#1077](https://github.com/ordinals/ord/pull/1077) by [casey](https://github.com/casey))
- Make inscription text white on inscription page ([#1079](https://github.com/ordinals/ord/pull/1079) by [casey](https://github.com/casey))
- Move templates into root module ([#1090](https://github.com/ordinals/ord/pull/1090) by [casey](https://github.com/casey))
- Show text inscriptions on homepage ([#1058](https://github.com/ordinals/ord/pull/1058) by [casey](https://github.com/casey))
- Add white background to inscriptions ([#1054](https://github.com/ordinals/ord/pull/1054) by [casey](https://github.com/casey))
- Show rare sat locations on /sat ([#1029](https://github.com/ordinals/ord/pull/1029) by [casey](https://github.com/casey))
- Add first signet inscription height ([#1016](https://github.com/ordinals/ord/pull/1016) by [casey](https://github.com/casey))
- Improve inscription style ([#1025](https://github.com/ordinals/ord/pull/1025) by [casey](https://github.com/casey))
- Improve ord-dev recipes ([#1022](https://github.com/ordinals/ord/pull/1022) by [casey](https://github.com/casey))
- Move inscription content above details ([#1017](https://github.com/ordinals/ord/pull/1017) by [casey](https://github.com/casey))
- Style latest inscriptions ([#1018](https://github.com/ordinals/ord/pull/1018) by [casey](https://github.com/casey))
- Print server URLs on startup ([#1015](https://github.com/ordinals/ord/pull/1015) by [casey](https://github.com/casey))
- Add inscription page preview image ([#1010](https://github.com/ordinals/ord/pull/1010) by [raphjaph](https://github.com/raphjaph))
- Show most recent inscriptions first on homepage and inscriptions page ([#1011](https://github.com/ordinals/ord/pull/1011) by [casey](https://github.com/casey))
- Display graphical inscriptions on homepage ([#1008](https://github.com/ordinals/ord/pull/1008) by [casey](https://github.com/casey))
- Add inscriptions page ([#1009](https://github.com/ordinals/ord/pull/1009) by [raphjaph](https://github.com/raphjaph))
- Minimize transaction fetching ([#1002](https://github.com/ordinals/ord/pull/1002) by [casey](https://github.com/casey))
- Rename `ord wallet satoshis` to `ord wallet sats` ([#1004](https://github.com/ordinals/ord/pull/1004) by [casey](https://github.com/casey))
- Update introduction.md ([#1000](https://github.com/ordinals/ord/pull/1000) by [batcavekid](https://github.com/batcavekid))
- Improve latest inscriptions style ([#999](https://github.com/ordinals/ord/pull/999) by [raphjaph](https://github.com/raphjaph))
- Show latest inscriptions on home page ([#996](https://github.com/ordinals/ord/pull/996) by [raphjaph](https://github.com/raphjaph))
- Add link to docs in readme ([#995](https://github.com/ordinals/ord/pull/995) by [casey](https://github.com/casey))
- Add inscription docs ([#994](https://github.com/ordinals/ord/pull/994) by [casey](https://github.com/casey))
- Fix softprops/actions-gh-release version ([#992](https://github.com/ordinals/ord/pull/992) by [casey](https://github.com/casey))
- Fuzz test transaction builder with multiple UTXOs ([#1291](https://github.com/ordinals/ord/pull/1291) by [casey](https://github.com/casey))

[0.3.0](https://github.com/ordinals/ord/releases/tag/0.3.0) - 2022-12-16
---------------------------------------------------------------------

- Update CI dependencies ([#986](https://github.com/ordinals/ord/pull/986) by [casey](https://github.com/casey))
- Add /content endpoint ([#976](https://github.com/ordinals/ord/pull/976) by [casey](https://github.com/casey))
- Display content type and size /inscription ([#975](https://github.com/ordinals/ord/pull/975) by [casey](https://github.com/casey))
- Use "sat" in place of "ordinal" ([#979](https://github.com/ordinals/ord/pull/979) by [casey](https://github.com/casey))

[0.2.1](https://github.com/ordinals/ord/releases/tag/0.2.1) - 2022-12-14
---------------------------------------------------------------------

- Revise inscription guide after mainnet walkthrough ([#968](https://github.com/ordinals/ord/pull/968) by [casey](https://github.com/casey))

[0.2.0](https://github.com/ordinals/ord/releases/tag/0.2.0) - 2022-12-14
---------------------------------------------------------------------

- Add `ord wallet create` ([#958](https://github.com/ordinals/ord/pull/958) by [casey](https://github.com/casey))
- Add chain flags ([#961](https://github.com/ordinals/ord/pull/961) by [casey](https://github.com/casey))
- Make inscription parser more lenient ([#956](https://github.com/ordinals/ord/pull/956) by [casey](https://github.com/casey))
- Add `ord wallet transactions` ([#951](https://github.com/ordinals/ord/pull/951) by [raphjaph](https://github.com/raphjaph))
- Update dependencies ([#955](https://github.com/ordinals/ord/pull/955) by [casey](https://github.com/casey))
- Show inscription on reveal transaction page ([#954](https://github.com/ordinals/ord/pull/954) by [casey](https://github.com/casey))
- Mention that wallet may not need to be loaded ([#953](https://github.com/ordinals/ord/pull/953) by [casey](https://github.com/casey))
- Document install script ([#952](https://github.com/ordinals/ord/pull/952) by [casey](https://github.com/casey))
- Revise homepage ([#950](https://github.com/ordinals/ord/pull/950) by [casey](https://github.com/casey))
- Add content to guide page ([#945](https://github.com/ordinals/ord/pull/945) by [casey](https://github.com/casey))
- Mention incentive to run full node in FAQ ([#948](https://github.com/ordinals/ord/pull/948) by [casey](https://github.com/casey))
- Expand FAQ ([#944](https://github.com/ordinals/ord/pull/944) by [casey](https://github.com/casey))
- Change --index-ordinals to --index-satoshis ([#942](https://github.com/ordinals/ord/pull/942) by [casey](https://github.com/casey))
- Improve wording ([#938](https://github.com/ordinals/ord/pull/938) by [casey](https://github.com/casey))
- Add help text to subcommands ([#934](https://github.com/ordinals/ord/pull/934) by [casey](https://github.com/casey))
- Merge CI jobs into one workflow ([#935](https://github.com/ordinals/ord/pull/935) by [casey](https://github.com/casey))
- Add install script ([#940](https://github.com/ordinals/ord/pull/940) by [casey](https://github.com/casey))
- Build MacOS ARM Binaries ([#941](https://github.com/ordinals/ord/pull/941) by [casey](https://github.com/casey))
- Add inscription guide ([#912](https://github.com/ordinals/ord/pull/912) by [casey](https://github.com/casey))
- Allow inscribing without specifying a satpoint ([#919](https://github.com/ordinals/ord/pull/919) by [raphjaph](https://github.com/raphjaph))
- Add `ord wallet inscriptions` ([#917](https://github.com/ordinals/ord/pull/917) by [raphjaph](https://github.com/raphjaph))
- Add `ord wallet utxos` ([#911](https://github.com/ordinals/ord/pull/911) by [raphjaph](https://github.com/raphjaph))
- Add `ord wallet recieve` ([#909](https://github.com/ordinals/ord/pull/909) by [raphjaph](https://github.com/raphjaph))
- Fix signet block explorer link ([#908](https://github.com/ordinals/ord/pull/908) by [casey](https://github.com/casey))
- Opt wallet transactions into RBF ([#901](https://github.com/ordinals/ord/pull/901) by [casey](https://github.com/casey))
- Avoid `as` conversions ([#903](https://github.com/ordinals/ord/pull/903) by [casey](https://github.com/casey))
- Save commit transaction recovery key ([#885](https://github.com/ordinals/ord/pull/885) by [raphjaph](https://github.com/raphjaph))
- Refuse to send inscriptions by satpoint ([#898](https://github.com/ordinals/ord/pull/898) by [casey](https://github.com/casey))
- Limit inscription content to 1024 bytes on signet and testnet ([#896](https://github.com/ordinals/ord/pull/896) by [casey](https://github.com/casey))
- Extend bounty 3 ([#897](https://github.com/ordinals/ord/pull/897) by [casey](https://github.com/casey))
- Make inscription type more flexible ([#892](https://github.com/ordinals/ord/pull/892) by [casey](https://github.com/casey))
- Update dependencies ([#894](https://github.com/ordinals/ord/pull/894) by [casey](https://github.com/casey))
- Refuse to inscribe UTXOs with additional inscriptions ([#880](https://github.com/ordinals/ord/pull/880) by [raphjaph](https://github.com/raphjaph))
- Make inscriptions support backwards-compatible extension ([#888](https://github.com/ordinals/ord/pull/888) by [casey](https://github.com/casey))
- Refuse to send additional inscriptions ([#881](https://github.com/ordinals/ord/pull/881) by [raphjaph](https://github.com/raphjaph))
- Enable Windows tests on CI ([#846](https://github.com/ordinals/ord/pull/846) by [casey](https://github.com/casey))
- Refuse to inscribe sats that have already been inscribe ([#878](https://github.com/ordinals/ord/pull/878) by [raphjaph](https://github.com/raphjaph))
- Send by inscription ID ([#877](https://github.com/ordinals/ord/pull/877) by [casey](https://github.com/casey))
- Test commands which return errors when not tracking rare ordinals ([#875](https://github.com/ordinals/ord/pull/875) by [casey](https://github.com/casey))
- Don't store serialized inscriptions ([#872](https://github.com/ordinals/ord/pull/872) by [casey](https://github.com/casey))
- Do not select inscribed sats as cardinal utxos ([#835](https://github.com/ordinals/ord/pull/835) by [raphjaph](https://github.com/raphjaph))
- Make ord info work without ordinal index ([#874](https://github.com/ordinals/ord/pull/874) by [casey](https://github.com/casey))
- Improve subcommand names ([#867](https://github.com/ordinals/ord/pull/867) by [casey](https://github.com/casey))
- Calculate TXIDs in background thread ([#866](https://github.com/ordinals/ord/pull/866) by [casey](https://github.com/casey))
- Track inscription satpoints ([#860](https://github.com/ordinals/ord/pull/860) by [raphjaph](https://github.com/raphjaph))
- Add type aliases index for array types ([#859](https://github.com/ordinals/ord/pull/859) by [casey](https://github.com/casey))
- Index inscriptions when not indexing ordinals ([#857](https://github.com/ordinals/ord/pull/857) by [casey](https://github.com/casey))
- Use satpoints instead of ordinals in wallet commands ([#849](https://github.com/ordinals/ord/pull/849) by [raphjaph](https://github.com/raphjaph))
- Only request transactions if indexing ordinals ([#851](https://github.com/ordinals/ord/pull/851) by [casey](https://github.com/casey))
- Make analyzing index easier ([#850](https://github.com/ordinals/ord/pull/850) by [casey](https://github.com/casey))
- Add `ord list-ranges <OUTPOINT>` ([#848](https://github.com/ordinals/ord/pull/848) by [raphjaph](https://github.com/raphjaph))
- Conditionally disable ordinal index dependent server features ([#845](https://github.com/ordinals/ord/pull/845) by [casey](https://github.com/casey))
- Update redb ([#832](https://github.com/ordinals/ord/pull/832) by [casey](https://github.com/casey))
- Compress downloaded logs ([#836](https://github.com/ordinals/ord/pull/836) by [casey](https://github.com/casey))
- Only index ordinal ranges if `--index-ordinals` is passed ([#837](https://github.com/ordinals/ord/pull/837) by [casey](https://github.com/casey))
- Record commit block count and timestamp in index ([#826](https://github.com/ordinals/ord/pull/826) by [casey](https://github.com/casey))
- Add build-snapshots recipe ([#831](https://github.com/ordinals/ord/pull/831) by [casey](https://github.com/casey))
- Add minimum system requirements to readme ([#829](https://github.com/ordinals/ord/pull/829) by [casey](https://github.com/casey))
- Abort update if another has run concurrently ([#830](https://github.com/ordinals/ord/pull/830) by [casey](https://github.com/casey))
- Add benchmark-revision recipe ([#827](https://github.com/ordinals/ord/pull/827) by [casey](https://github.com/casey))
- Retry get_block_hash as well as get_block ([#820](https://github.com/ordinals/ord/pull/820) by [casey](https://github.com/casey))
- Update dependencies ([#823](https://github.com/ordinals/ord/pull/823) by [casey](https://github.com/casey))
- Add inscription page ([#817](https://github.com/ordinals/ord/pull/817) by [raphjaph](https://github.com/raphjaph))
- Add PNG inscriptions ([#800](https://github.com/ordinals/ord/pull/800) by [raphjaph](https://github.com/raphjaph))
- Disable inscriptions on mainnet ([#814](https://github.com/ordinals/ord/pull/814) by [casey](https://github.com/casey))
- Add benchmark recipe ([#810](https://github.com/ordinals/ord/pull/810) by [casey](https://github.com/casey))
- Display chain in header if not on mainnet ([#809](https://github.com/ordinals/ord/pull/809) by [casey](https://github.com/casey))
- Show difficulty target on block page ([#750](https://github.com/ordinals/ord/pull/750) by [casey](https://github.com/casey))
- Deduct fee before calculating reveal transaction signature ([#780](https://github.com/ordinals/ord/pull/780) by [casey](https://github.com/casey))
- Remove redundant wallet balance check ([#764](https://github.com/ordinals/ord/pull/764) by [casey](https://github.com/casey))
- Add `ord wallet inscribe` command ([#658](https://github.com/ordinals/ord/pull/658) by [casey](https://github.com/casey))
- Remove outdated runes and inscriptions ([#760](https://github.com/ordinals/ord/pull/760) by [casey](https://github.com/casey))
- Prevent progress bar from flickering when synced ([#759](https://github.com/ordinals/ord/pull/759) by [casey](https://github.com/casey))
- Fix graph command to work with new format ([#755](https://github.com/ordinals/ord/pull/755) by [casey](https://github.com/casey))
- Track ordinal ranges ([#756](https://github.com/ordinals/ord/pull/756) by [casey](https://github.com/casey))
- Use HTTP connection reusing `rust-jsonrpc` ([#754](https://github.com/ordinals/ord/pull/754) by [casey](https://github.com/casey))
- Extend bounty 3 by one difficulty adjustment period ([#753](https://github.com/ordinals/ord/pull/753) by [casey](https://github.com/casey))
- Replace binary search in epoch construction ([#723](https://github.com/ordinals/ord/pull/723) by [veryordinally](https://github.com/veryordinally))
- Search for ordinals in TSV using `ord wallet identify` ([#729](https://github.com/ordinals/ord/pull/729) by [raphjaph](https://github.com/raphjaph))
- Don't create acme cache dir ([#727](https://github.com/ordinals/ord/pull/727) by [casey](https://github.com/casey))
- Split up ci into test and lint workflows ([#728](https://github.com/ordinals/ord/pull/728) by [casey](https://github.com/casey))
- Enable CI for Windows ([#603](https://github.com/ordinals/ord/pull/603) by [casey](https://github.com/casey))
- Add bounty 3 ([#725](https://github.com/ordinals/ord/pull/725) by [casey](https://github.com/casey))
- Fetch blocks in background ([#495](https://github.com/ordinals/ord/pull/495) by [casey](https://github.com/casey))
- Don't call `apt-get update` in CI workflow ([#719](https://github.com/ordinals/ord/pull/719) by [casey](https://github.com/casey))
- Remove old recipes from justfile ([#718](https://github.com/ordinals/ord/pull/718) by [casey](https://github.com/casey))
- Update roadmap ([#722](https://github.com/ordinals/ord/pull/722) by [raphjaph](https://github.com/raphjaph))

[0.1.0](https://github.com/ordinals/ord/releases/tag/0.1.0) - 2022-10-25
---------------------------------------------------------------------

- Add index updater ([#703](https://github.com/ordinals/ord/pull/703) by [veryordinally](https://github.com/veryordinally))
- Speed up rarity check while indexing ([#702](https://github.com/ordinals/ord/pull/702) by [veryordinally](https://github.com/veryordinally))

[0.0.6](https://github.com/ordinals/ord/releases/tag/0.0.6) - 2022-10-25
---------------------------------------------------------------------

- Switch to ord-bitcoincore-rpc ([#707](https://github.com/ordinals/ord/pull/707) by [casey](https://github.com/casey))
- Start error messages with lowercase character ([#693](https://github.com/ordinals/ord/pull/693) by [raphjaph](https://github.com/raphjaph))
- Ensure addresses are valid for network ([#698](https://github.com/ordinals/ord/pull/698) by [casey](https://github.com/casey))
- Link videos from docs ([#696](https://github.com/ordinals/ord/pull/696) by [casey](https://github.com/casey))
- Restrict `ord wallet send` on mainnet ([#687](https://github.com/ordinals/ord/pull/687) by [casey](https://github.com/casey))
- Improve progress bar ([#694](https://github.com/ordinals/ord/pull/694) by [casey](https://github.com/casey))
- Note bounty 2 has been claimed ([#700](https://github.com/ordinals/ord/pull/700) by [casey](https://github.com/casey))
- Don't opt-in to RBF ([#685](https://github.com/ordinals/ord/pull/685) by [raphjaph](https://github.com/raphjaph))
- Don't unintentionally send rare ordinals ([#683](https://github.com/ordinals/ord/pull/683) by [raphjaph](https://github.com/raphjaph))
- Enforce transaction construction output address invariants ([#682](https://github.com/ordinals/ord/pull/682) by [casey](https://github.com/casey))
- Use worst-case fee estimates ([#681](https://github.com/ordinals/ord/pull/681) by [casey](https://github.com/casey))
- Add encoding to clock SVG ([#678](https://github.com/ordinals/ord/pull/678) by [casey](https://github.com/casey))
- Add helpers to make transaction builder tests more concise ([#679](https://github.com/ordinals/ord/pull/679) by [casey](https://github.com/casey))
- Don't use UTXOs with rare ordinals as cardinal inputs ([#680](https://github.com/ordinals/ord/pull/680) by [casey](https://github.com/casey))
- Improve not enough cardinal UTXOs error message ([#675](https://github.com/ordinals/ord/pull/675) by [casey](https://github.com/casey))
- Pad initial output to be above dust limit ([#674](https://github.com/ordinals/ord/pull/674) by [casey](https://github.com/casey))
- Start indexing progress bar at current height ([#673](https://github.com/ordinals/ord/pull/673) by [casey](https://github.com/casey))
- Add additional postage when necessary ([#671](https://github.com/ordinals/ord/pull/671) by [casey](https://github.com/casey))
- Check transaction fees in transaction builder ([#669](https://github.com/ordinals/ord/pull/669) by [casey](https://github.com/casey))
- Display progress bar when indexing ([#668](https://github.com/ordinals/ord/pull/668) by [casey](https://github.com/casey))
- Send ordinal first in recipient output ([#666](https://github.com/ordinals/ord/pull/666) by [raphjaph](https://github.com/raphjaph))
- Add doc-comment to transaction builder ([#663](https://github.com/ordinals/ord/pull/663) by [casey](https://github.com/casey))
- Change feerate to 1 sat/vbyte ([#664](https://github.com/ordinals/ord/pull/664) by [raphjaph](https://github.com/raphjaph))
- Strip excess postage from end of output ([#662](https://github.com/ordinals/ord/pull/662) by [raphjaph](https://github.com/raphjaph))
- Download logs to tempdir ([#656](https://github.com/ordinals/ord/pull/656) by [casey](https://github.com/casey))
- Improve transaction builder checks ([#661](https://github.com/ordinals/ord/pull/661) by [casey](https://github.com/casey))
- Use redb's two-phase write strategy in production ([#660](https://github.com/ordinals/ord/pull/660) by [casey](https://github.com/casey))
- Replace `Result<()>` with `Result` ([#657](https://github.com/ordinals/ord/pull/657) by [casey](https://github.com/casey))
- Add fee when sending ([#655](https://github.com/ordinals/ord/pull/655) by [raphjaph](https://github.com/raphjaph))
- Make table names more explicit ([#654](https://github.com/ordinals/ord/pull/654) by [casey](https://github.com/casey))
- Fix race condition in commit test ([#651](https://github.com/ordinals/ord/pull/651) by [casey](https://github.com/casey))
- Reform `ord wallet send` ([#648](https://github.com/ordinals/ord/pull/648) by [raphjaph](https://github.com/raphjaph))
- Use https://signet.ordinals.com as default signet publish URL ([#649](https://github.com/ordinals/ord/pull/649) by [casey](https://github.com/casey))
- Append network to data dir ([#650](https://github.com/ordinals/ord/pull/650) by [casey](https://github.com/casey))
- Only commit when necessary ([#647](https://github.com/ordinals/ord/pull/647) by [casey](https://github.com/casey))
- Make rarity text white ([#644](https://github.com/ordinals/ord/pull/644) by [casey](https://github.com/casey))
- Link to ordinal from rune ([#643](https://github.com/ordinals/ord/pull/643) by [casey](https://github.com/casey))
- Show inscriptions on /ordinal ([#645](https://github.com/ordinals/ord/pull/645) by [casey](https://github.com/casey))
- Document search ([#646](https://github.com/ordinals/ord/pull/646) by [casey](https://github.com/casey))
- Check that RPC server is on correct network ([#642](https://github.com/ordinals/ord/pull/642) by [casey](https://github.com/casey))
- Add /input page ([#639](https://github.com/ordinals/ord/pull/639) by [casey](https://github.com/casey))
- Expand search box to fill available space ([#633](https://github.com/ordinals/ord/pull/633) by [casey](https://github.com/casey))
- Add `ord rune publish` command ([#637](https://github.com/ordinals/ord/pull/637) by [casey](https://github.com/casey))
- Add links to docs ([#635](https://github.com/ordinals/ord/pull/635) by [casey](https://github.com/casey))
- Use docs for name of workflow and directory ([#632](https://github.com/ordinals/ord/pull/632) by [casey](https://github.com/casey))
- Remove multilingual book config key ([#631](https://github.com/ordinals/ord/pull/631) by [casey](https://github.com/casey))
- Add `ord wallet send` ([#618](https://github.com/ordinals/ord/pull/618) by [raphjaph](https://github.com/raphjaph))
- Streamline roadmap ([#628](https://github.com/ordinals/ord/pull/628) by [casey](https://github.com/casey))
- Improve styling ([#626](https://github.com/ordinals/ord/pull/626) by [casey](https://github.com/casey))
- Fix book publish directory ([#625](https://github.com/ordinals/ord/pull/625) by [casey](https://github.com/casey))
- Convert docs from Zola to mdBook ([#623](https://github.com/ordinals/ord/pull/623) by [casey](https://github.com/casey))
- Add nav bar ([#614](https://github.com/ordinals/ord/pull/614) by [casey](https://github.com/casey))
- Add status header to homepage ([#620](https://github.com/ordinals/ord/pull/620) by [casey](https://github.com/casey))
- Update roadmap ([#617](https://github.com/ordinals/ord/pull/617) by [casey](https://github.com/casey))
- Use reduced database durability during tests ([#621](https://github.com/ordinals/ord/pull/621) by [casey](https://github.com/casey))
- Add /rare.txt ([#619](https://github.com/ordinals/ord/pull/619) by [casey](https://github.com/casey))
- Embellish block page ([#605](https://github.com/ordinals/ord/pull/605) by [casey](https://github.com/casey))
- Refactor server error handling ([#607](https://github.com/ordinals/ord/pull/607) by [casey](https://github.com/casey))
- Profile tests ([#608](https://github.com/ordinals/ord/pull/608) by [casey](https://github.com/casey))
- Display ranges with an en dash ([#606](https://github.com/ordinals/ord/pull/606) by [casey](https://github.com/casey))
- Display more information homepage ([#610](https://github.com/ordinals/ord/pull/610) by [casey](https://github.com/casey))
- Remove prime trait ([#612](https://github.com/ordinals/ord/pull/612) by [casey](https://github.com/casey))
- Sort ordinal properties ([#609](https://github.com/ordinals/ord/pull/609) by [casey](https://github.com/casey))
- Add dark mode ([#611](https://github.com/ordinals/ord/pull/611) by [casey](https://github.com/casey))
- Add more help text to CLI ([#613](https://github.com/ordinals/ord/pull/613) by [casey](https://github.com/casey))
- Expand ordinal hunting guide ([#600](https://github.com/ordinals/ord/pull/600) by [casey](https://github.com/casey))
- Embellish transaction page ([#602](https://github.com/ordinals/ord/pull/602) by [casey](https://github.com/casey))
- Add `ord wallet list` command ([#601](https://github.com/ordinals/ord/pull/601) by [raphjaph](https://github.com/raphjaph))
- Ignore temporary directory ([#594](https://github.com/ordinals/ord/pull/594) by [casey](https://github.com/casey))
- Add ordinal hunting how-to docs page ([#596](https://github.com/ordinals/ord/pull/596) by [raphjaph](https://github.com/raphjaph))
- Fix bounty example links ([#595](https://github.com/ordinals/ord/pull/595) by [casey](https://github.com/casey))

[0.0.5](https://github.com/ordinals/ord/releases/tag/0.0.5) - 2022-10-02
---------------------------------------------------------------------

- Add bitcoin.conf ([#592](https://github.com/ordinals/ord/pull/592) by [casey](https://github.com/casey))
- Add uncommon ordinal bounty ([#588](https://github.com/ordinals/ord/pull/588) by [casey](https://github.com/casey))
- Show output size on output page ([#590](https://github.com/ordinals/ord/pull/590) by [casey](https://github.com/casey))
- Implement `wallet identify` ([#586](https://github.com/ordinals/ord/pull/586) by [raphjaph](https://github.com/raphjaph))
- Report integration test times ([#587](https://github.com/ordinals/ord/pull/587) by [casey](https://github.com/casey))
- Show message when output couldn't be listed because it was spent ([#585](https://github.com/ordinals/ord/pull/585) by [casey](https://github.com/casey))
- Add server integration test ([#583](https://github.com/ordinals/ord/pull/583) by [casey](https://github.com/casey))
- Use constants from rust-bitcoin ([#564](https://github.com/ordinals/ord/pull/564) by [casey](https://github.com/casey))
- Update dependencies ([#582](https://github.com/ordinals/ord/pull/582) by [casey](https://github.com/casey))
- Move bounties into subpages ([#576](https://github.com/ordinals/ord/pull/576) by [casey](https://github.com/casey))
- Convert last find integration test to unit test ([#580](https://github.com/ordinals/ord/pull/580) by [raphjaph](https://github.com/raphjaph))
- Make index::custom_index_size test faster ([#579](https://github.com/ordinals/ord/pull/579) by [casey](https://github.com/casey))
- Make info::basic test faster ([#578](https://github.com/ordinals/ord/pull/578) by [casey](https://github.com/casey))
- Convert list unit tests to inegration tests ([#572](https://github.com/ordinals/ord/pull/572) by [raphjaph](https://github.com/raphjaph))
- Add prime trait ([#563](https://github.com/ordinals/ord/pull/563) by [casey](https://github.com/casey))
- Rename workflow jobs ([#575](https://github.com/ordinals/ord/pull/575) by [casey](https://github.com/casey))
- Convert some find integration tests to unit tests ([#571](https://github.com/ordinals/ord/pull/571) by [casey](https://github.com/casey))
- Remove /clock.svg route ([#573](https://github.com/ordinals/ord/pull/573) by [casey](https://github.com/casey))
- Fix test bitcoin core rpc server compilation ([#570](https://github.com/ordinals/ord/pull/570) by [casey](https://github.com/casey))
- Move test Bitcoin Core RPC server into sub-crate ([#569](https://github.com/ordinals/ord/pull/569) by [casey](https://github.com/casey))
- Remove spent output test ([#568](https://github.com/ordinals/ord/pull/568) by [casey](https://github.com/casey))
- Remove find-by-slot tests ([#567](https://github.com/ordinals/ord/pull/567) by [casey](https://github.com/casey))
- Remove BDK wallet ([#566](https://github.com/ordinals/ord/pull/566) by [casey](https://github.com/casey))
- Show if a reorg has happened on /status ([#518](https://github.com/ordinals/ord/pull/518) by [raphjaph](https://github.com/raphjaph))
- Convert block and transaction integration tests to unit tests ([#560](https://github.com/ordinals/ord/pull/560) by [raphjaph](https://github.com/raphjaph))
- Fix release script ([#562](https://github.com/ordinals/ord/pull/562) by [casey](https://github.com/casey))

[0.0.4](https://github.com/ordinals/ord/releases/tag/0.0.4) - 2022-09-26
---------------------------------------------------------------------

- Add more links and labels to clocks ([#552](https://github.com/ordinals/ord/pull/552) by [casey](https://github.com/casey))
- Add script to deploy dev server on production machines ([#550](https://github.com/ordinals/ord/pull/550) by [casey](https://github.com/casey))
- Update redb to 0.7.0 ([#559](https://github.com/ordinals/ord/pull/559) by [windsok](https://github.com/windsok))
- Don't block server on index ([#551](https://github.com/ordinals/ord/pull/551) by [casey](https://github.com/casey))
- Allow searching for block hashes, txids, and outputs things ([#549](https://github.com/ordinals/ord/pull/549) by [casey](https://github.com/casey))
- Convert more integration tests to unit tests ([#548](https://github.com/ordinals/ord/pull/548) by [raphjaph](https://github.com/raphjaph))
- Make range integration tests faster ([#547](https://github.com/ordinals/ord/pull/547) by [casey](https://github.com/casey))
- Add roadmap ([#546](https://github.com/ordinals/ord/pull/546) by [raphjaph](https://github.com/raphjaph))
- Convert some integration tests to unit tests ([#544](https://github.com/ordinals/ord/pull/544) by [raphjaph](https://github.com/raphjaph))
- Sync index on `Index::open` ([#545](https://github.com/ordinals/ord/pull/545) by [casey](https://github.com/casey))
- Make some tests faster ([#543](https://github.com/ordinals/ord/pull/543) by [casey](https://github.com/casey))
- Add search-by-path endpoint at /search/QUERY ([#521](https://github.com/ordinals/ord/pull/521) by [raphjaph](https://github.com/raphjaph))
- Note why unit tests should use regtest network ([#539](https://github.com/ordinals/ord/pull/539) by [casey](https://github.com/casey))
- Use --chain regtest to speed up unit tests ([#538](https://github.com/ordinals/ord/pull/538) by [casey](https://github.com/casey))
- Add attributes to search box ([#520](https://github.com/ordinals/ord/pull/520) by [casey](https://github.com/casey))
- Fix off-by-some --height-limit bug ([#526](https://github.com/ordinals/ord/pull/526) by [casey](https://github.com/casey))
- Count total number of outputs traversed when building index ([#525](https://github.com/ordinals/ord/pull/525) by [raphjaph](https://github.com/raphjaph))
- Use boilerplate 0.2.0 ([#531](https://github.com/ordinals/ord/pull/531) by [casey](https://github.com/casey))
- Add favicon to docs.ordinals.com ([#530](https://github.com/ordinals/ord/pull/530) by [casey](https://github.com/casey))
- Move docs to GitHub Pages ([#515](https://github.com/ordinals/ord/pull/515) by [casey](https://github.com/casey))
- Bounty 1 claimed! ([#529](https://github.com/ordinals/ord/pull/529) by [casey](https://github.com/casey))
- Use fixed-size index keys and values. ([#516](https://github.com/ordinals/ord/pull/516) by [casey](https://github.com/casey))
- Update dependencies ([#519](https://github.com/ordinals/ord/pull/519) by [casey](https://github.com/casey))
- Log retry interval ([#509](https://github.com/ordinals/ord/pull/509) by [veryordinally](https://github.com/veryordinally))
- Retry with exponential backoff on RPC errors during indexing ([#508](https://github.com/ordinals/ord/pull/508) by [casey](https://github.com/casey))
- Include outpoint in missing outpoint message ([#506](https://github.com/ordinals/ord/pull/506) by [casey](https://github.com/casey))
- Link to clock from home page ([#499](https://github.com/ordinals/ord/pull/499) by [casey](https://github.com/casey))
- Pass benchmark dir name in justfile recipe ([#498](https://github.com/ordinals/ord/pull/498) by [casey](https://github.com/casey))
- Improve benchmark ([#497](https://github.com/ordinals/ord/pull/497) by [casey](https://github.com/casey))
- Commit every 1000 blocks instead of every block ([#496](https://github.com/ordinals/ord/pull/496) by [casey](https://github.com/casey))
- Improve benchmark script ([#493](https://github.com/ordinals/ord/pull/493) by [casey](https://github.com/casey))
- Add colors and tooltips to clock ([#476](https://github.com/ordinals/ord/pull/476) by [raphjaph](https://github.com/raphjaph))
- Block height to clock ([#477](https://github.com/ordinals/ord/pull/477) by [raphjaph](https://github.com/raphjaph))
- Add benchmark script ([#488](https://github.com/ordinals/ord/pull/488) by [casey](https://github.com/casey))
- Add flamegraph recipe ([#486](https://github.com/ordinals/ord/pull/486) by [casey](https://github.com/casey))
- Fix degree parsing ([#485](https://github.com/ordinals/ord/pull/485) by [raphjaph](https://github.com/raphjaph))
- Add search box to homepage ([#479](https://github.com/ordinals/ord/pull/479) by [casey](https://github.com/casey))
- Add shell.nix ([#475](https://github.com/ordinals/ord/pull/475) by [jurraca](https://github.com/jurraca))
- Fix indentation in test-deploy recipe ([#474](https://github.com/ordinals/ord/pull/474) by [casey](https://github.com/casey))
- Document how to turn on logging ([#464](https://github.com/ordinals/ord/pull/464) by [casey](https://github.com/casey))
- Add contribution advice to readme ([#460](https://github.com/ordinals/ord/pull/460) by [casey](https://github.com/casey))
- Increase default maximum index size for non-regtest chains ([#448](https://github.com/ordinals/ord/pull/448) by [casey](https://github.com/casey))
- Remove old NFT mint and verify commands ([#418](https://github.com/ordinals/ord/pull/418) by [casey](https://github.com/casey))
- Update readme ([#399](https://github.com/ordinals/ord/pull/399) by [casey](https://github.com/casey))
- Allow serving HTTP and HTTPS simultaneously ([#359](https://github.com/ordinals/ord/pull/359) by [casey](https://github.com/casey))
- Prevent ordinals that are being sent from being spent as fees ([#369](https://github.com/ordinals/ord/pull/369) by [terror](https://github.com/terror))
- Add error on None case for special_ordinals ([#382](https://github.com/ordinals/ord/pull/382) by [terror](https://github.com/terror))
- Guard against invalid percentiles ([#380](https://github.com/ordinals/ord/pull/380) by [casey](https://github.com/casey))
- Add percentile representation ([#378](https://github.com/ordinals/ord/pull/378) by [casey](https://github.com/casey))
- Make --acme-contact optional ([#379](https://github.com/ordinals/ord/pull/379) by [casey](https://github.com/casey))
- Improve names for a couple of properties ([#377](https://github.com/ordinals/ord/pull/377) by [casey](https://github.com/casey))
- [bin/graph] Skip previous syncs ([#376](https://github.com/ordinals/ord/pull/376) by [casey](https://github.com/casey))
- Add graph recipe ([#375](https://github.com/ordinals/ord/pull/375) by [terror](https://github.com/terror))
- Log ord by default ([#374](https://github.com/ordinals/ord/pull/374) by [casey](https://github.com/casey))
- Don't write to OUTPOINT_TO_TXID table ([#373](https://github.com/ordinals/ord/pull/373) by [casey](https://github.com/casey))
- Change just recipe to log main instance by default ([#372](https://github.com/ordinals/ord/pull/372) by [casey](https://github.com/casey))
- Add bounty 1 ([#370](https://github.com/ordinals/ord/pull/370) by [casey](https://github.com/casey))
- Don't hardcode cookie file in deploy script ([#367](https://github.com/ordinals/ord/pull/367) by [casey](https://github.com/casey))
- Remove comments from service files ([#368](https://github.com/ordinals/ord/pull/368) by [casey](https://github.com/casey))
- Add special ordinal protection ([#357](https://github.com/ordinals/ord/pull/357) by [terror](https://github.com/terror))
- Add defaults for --acme-cache and --acme-domain ([#364](https://github.com/ordinals/ord/pull/364) by [casey](https://github.com/casey))
- Read cookie file from --bitcoin-data-dir ([#365](https://github.com/ordinals/ord/pull/365) by [casey](https://github.com/casey))
- Pass network to deploy scripts ([#366](https://github.com/ordinals/ord/pull/366) by [casey](https://github.com/casey))
- Put .hushlogin in correct location ([#363](https://github.com/ordinals/ord/pull/363) by [casey](https://github.com/casey))
- Pass domain to deploy scripts ([#361](https://github.com/ordinals/ord/pull/361) by [casey](https://github.com/casey))
- Suppress login messages ([#360](https://github.com/ordinals/ord/pull/360) by [casey](https://github.com/casey))
- Disable password auth on deploy ([#358](https://github.com/ordinals/ord/pull/358) by [casey](https://github.com/casey))
- Improve deploy scripts ([#342](https://github.com/ordinals/ord/pull/342) by [casey](https://github.com/casey))
- Tick tock next block ([#355](https://github.com/ordinals/ord/pull/355) by [casey](https://github.com/casey))
- Add `ord wallet identify` ([#304](https://github.com/ordinals/ord/pull/304) by [terror](https://github.com/terror))
- Note bounty #0 has been claimed ([#356](https://github.com/ordinals/ord/pull/356) by [casey](https://github.com/casey))
- Remove unused CSS font-family ([#354](https://github.com/ordinals/ord/pull/354) by [casey](https://github.com/casey))
- Use rustl-acme acceptor ([#289](https://github.com/ordinals/ord/pull/289) by [casey](https://github.com/casey))
- Display hashes, ranges, and outputs in monospace ([#353](https://github.com/ordinals/ord/pull/353) by [casey](https://github.com/casey))
- Improve <ol> style ([#352](https://github.com/ordinals/ord/pull/352) by [casey](https://github.com/casey))
- Add temporary favicon ([#351](https://github.com/ordinals/ord/pull/351) by [casey](https://github.com/casey))
- Make deploys faster ([#350](https://github.com/ordinals/ord/pull/350) by [casey](https://github.com/casey))
- Color blocks on homepage by rarity ([#349](https://github.com/ordinals/ord/pull/349) by [terror](https://github.com/terror))
- Rarity-color ranges in outputs and link to first ordinal in ranges ([#348](https://github.com/ordinals/ord/pull/348) by [casey](https://github.com/casey))
- Remove slide deck ([#346](https://github.com/ordinals/ord/pull/346) by [casey](https://github.com/casey))
- Switch to one-at-a-time bounties ([#347](https://github.com/ordinals/ord/pull/347) by [casey](https://github.com/casey))
- Add better message for spent outputs ([#345](https://github.com/ordinals/ord/pull/345) by [terror](https://github.com/terror))
- Use <ol> for homepage ([#343](https://github.com/ordinals/ord/pull/343) by [casey](https://github.com/casey))
- Remove GitHub pages directory ([#344](https://github.com/ordinals/ord/pull/344) by [casey](https://github.com/casey))
- Rename / page from "root" to "home" ([#341](https://github.com/ordinals/ord/pull/341) by [casey](https://github.com/casey))
- Remove sleeps from server tests ([#340](https://github.com/ordinals/ord/pull/340) by [casey](https://github.com/casey))
- Add space around nav items ([#338](https://github.com/ordinals/ord/pull/338) by [casey](https://github.com/casey))
- Style links ([#337](https://github.com/ordinals/ord/pull/337) by [casey](https://github.com/casey))
- Add FAQ and bounty ([#339](https://github.com/ordinals/ord/pull/339) by [casey](https://github.com/casey))
- Add links to homepage ([#335](https://github.com/ordinals/ord/pull/335) by [casey](https://github.com/casey))
- Styling ([#333](https://github.com/ordinals/ord/pull/333) by [casey](https://github.com/casey))
- Remove fluff from BIP ([#336](https://github.com/ordinals/ord/pull/336) by [casey](https://github.com/casey))
- Remove old comment from bitcoind.service ([#334](https://github.com/ordinals/ord/pull/334) by [casey](https://github.com/casey))
- Add viewport meta tag ([#332](https://github.com/ordinals/ord/pull/332) by [terror](https://github.com/terror))
- Add rarity colors ([#330](https://github.com/ordinals/ord/pull/330) by [casey](https://github.com/casey))
- Don't let ordinals become telephone numbers ([#331](https://github.com/ordinals/ord/pull/331) by [terror](https://github.com/terror))
- Add next and prev links to /ordinal ([#329](https://github.com/ordinals/ord/pull/329) by [terror](https://github.com/terror))
- Fix broken link ([#328](https://github.com/ordinals/ord/pull/328) by [casey](https://github.com/casey))
- Add header to /range ([#325](https://github.com/ordinals/ord/pull/325) by [casey](https://github.com/casey))
- Fix off by one bug in index::blocks ([#326](https://github.com/ordinals/ord/pull/326) by [terror](https://github.com/terror))
- Add header to /output ([#324](https://github.com/ordinals/ord/pull/324) by [casey](https://github.com/casey))
- Limit blocks ([#320](https://github.com/ordinals/ord/pull/320) by [casey](https://github.com/casey))
- Add header to /tx ([#322](https://github.com/ordinals/ord/pull/322) by [casey](https://github.com/casey))
- Add header to /block/HASH ([#321](https://github.com/ordinals/ord/pull/321) by [casey](https://github.com/casey))
- Convert / to boilerplate template ([#317](https://github.com/ordinals/ord/pull/317) by [casey](https://github.com/casey))
- Return BlockHash from Index::all ([#319](https://github.com/ordinals/ord/pull/319) by [casey](https://github.com/casey))
- Don't warn about installing bitcoind in deploy/setup ([#318](https://github.com/ordinals/ord/pull/318) by [casey](https://github.com/casey))
- Improvements ([#298](https://github.com/ordinals/ord/pull/298) by [casey](https://github.com/casey))
- Update rust toolchain when deploying ([#311](https://github.com/ordinals/ord/pull/311) by [casey](https://github.com/casey))
- Fix forbidden word check ([#313](https://github.com/ordinals/ord/pull/313) by [casey](https://github.com/casey))
- Don't run integration tests on MacOS CI ([#316](https://github.com/ordinals/ord/pull/316) by [casey](https://github.com/casey))
- Disable redb checksums ([#315](https://github.com/ordinals/ord/pull/315) by [casey](https://github.com/casey))
- Pay a fixed fee when sending transactions ([#314](https://github.com/ordinals/ord/pull/314) by [terror](https://github.com/terror))
- Refactor duplicate blockchain code in purse ([#312](https://github.com/ordinals/ord/pull/312) by [terror](https://github.com/terror))
- Add `ord wallet send` ([#305](https://github.com/ordinals/ord/pull/305) by [terror](https://github.com/terror))
- Add wallet balance subcommand ([#271](https://github.com/ordinals/ord/pull/271) by [terror](https://github.com/terror))
- Add wallet utxos subcommand ([#259](https://github.com/ordinals/ord/pull/259) by [terror](https://github.com/terror))
- Use bitcoin core node for integration tests ([#263](https://github.com/ordinals/ord/pull/263) by [terror](https://github.com/terror))
- List transaction outputs ([#292](https://github.com/ordinals/ord/pull/292) by [terror](https://github.com/terror))
- Add `/output/:outpoint` endpoint ([#293](https://github.com/ordinals/ord/pull/293) by [casey](https://github.com/casey))
- Add /range/:start/:end endpoint ([#291](https://github.com/ordinals/ord/pull/291) by [casey](https://github.com/casey))
- Move /list endpoint to /api/list ([#288](https://github.com/ordinals/ord/pull/288) by [casey](https://github.com/casey))
- List block transactions at `/block/:hash` ([#286](https://github.com/ordinals/ord/pull/286) by [terror](https://github.com/terror))
- Display ordinals at `/ordinal/:ordinal` ([#287](https://github.com/ordinals/ord/pull/287) by [casey](https://github.com/casey))
- Wait for bitcoind and ord to become available ([#285](https://github.com/ordinals/ord/pull/285) by [casey](https://github.com/casey))
- List blocks on root page ([#276](https://github.com/ordinals/ord/pull/276) by [terror](https://github.com/terror))
- Remove user-facing list page ([#275](https://github.com/ordinals/ord/pull/275) by [casey](https://github.com/casey))
- Add network option ([#274](https://github.com/ordinals/ord/pull/274) by [terror](https://github.com/terror))
- Serve HTTPS with ACME certs ([#256](https://github.com/ordinals/ord/pull/256) by [casey](https://github.com/casey))
- Remove unused functionality ([#270](https://github.com/ordinals/ord/pull/270) by [casey](https://github.com/casey))
- Revise homepage ([#268](https://github.com/ordinals/ord/pull/268) by [casey](https://github.com/casey))
- Link to blog post ([#267](https://github.com/ordinals/ord/pull/267) by [casey](https://github.com/casey))
- Use hour, minute, second, and third terminology ([#262](https://github.com/ordinals/ord/pull/262) by [casey](https://github.com/casey))
- Allow passing ordinals in degree and decimal notation ([#261](https://github.com/ordinals/ord/pull/261) by [casey](https://github.com/casey))
- Update dependencies ([#258](https://github.com/ordinals/ord/pull/258) by [casey](https://github.com/casey))
- Make genesis sat mythic ([#260](https://github.com/ordinals/ord/pull/260) by [casey](https://github.com/casey))
- Add wallet ([#233](https://github.com/ordinals/ord/pull/233) by [terror](https://github.com/terror))
- Overhaul traits ([#255](https://github.com/ordinals/ord/pull/255) by [casey](https://github.com/casey))
- Clarify duplicate transaction rule in BIP ([#254](https://github.com/ordinals/ord/pull/254) by [casey](https://github.com/casey))
- Purge LMDB ([#231](https://github.com/ordinals/ord/pull/231) by [casey](https://github.com/casey))
- Add justfile with commands for moving ordinals around manually ([#238](https://github.com/ordinals/ord/pull/238) by [casey](https://github.com/casey))
- Add links to discord server ([#237](https://github.com/ordinals/ord/pull/237) by [casey](https://github.com/casey))
- Make `nft verify` take input as argument ([#235](https://github.com/ordinals/ord/pull/235) by [casey](https://github.com/casey))
- Add --version flag ([#236](https://github.com/ordinals/ord/pull/236) by [casey](https://github.com/casey))
- Bump version: 0.0.2 → 0.0.3 ([#234](https://github.com/ordinals/ord/pull/234) by [casey](https://github.com/casey))
- Change deploy target in recipe ([#232](https://github.com/ordinals/ord/pull/232) by [terror](https://github.com/terror))
- Use default port and set ambient capabilities in ord service ([#230](https://github.com/ordinals/ord/pull/230) by [terror](https://github.com/terror))
- Test deploy on vagrant ([#229](https://github.com/ordinals/ord/pull/229) by [terror](https://github.com/terror))
- Update slide deck ([#227](https://github.com/ordinals/ord/pull/227) by [casey](https://github.com/casey))
- Add link to video ([#226](https://github.com/ordinals/ord/pull/226) by [casey](https://github.com/casey))
- Separate deck pages ([#225](https://github.com/ordinals/ord/pull/225) by [casey](https://github.com/casey))
- Fix docs HTML ([#224](https://github.com/ordinals/ord/pull/224) by [casey](https://github.com/casey))
- Add side deck ([#223](https://github.com/ordinals/ord/pull/223) by [casey](https://github.com/casey))
- Change slot notation to AxBxCxD ([#222](https://github.com/ordinals/ord/pull/222) by [casey](https://github.com/casey))
- Improve NFT encoding ([#221](https://github.com/ordinals/ord/pull/221) by [alok](https://github.com/alok))
- Remove use of sha256d in signature algorithm ([#219](https://github.com/ordinals/ord/pull/219) by [casey](https://github.com/casey))
- Use standard formats ([#218](https://github.com/ordinals/ord/pull/218) by [terror](https://github.com/terror))
- Use CBOR for serialization/deserialization ([#217](https://github.com/ordinals/ord/pull/217) by [terror](https://github.com/terror))
- Add nix flake ([#214](https://github.com/ordinals/ord/pull/214) by [jurraca](https://github.com/jurraca))
- Build binaries for releases ([#213](https://github.com/ordinals/ord/pull/213) by [casey](https://github.com/casey))

[0.0.1](https://github.com/ordinals/ord/releases/tag/0.0.1) - 2022-06-05
---------------------------------------------------------------------

- Add commands to mint and verify NFTs ([#211](https://github.com/ordinals/ord/pull/211) by [casey](https://github.com/casey))
- Add legendary sat location hints ([#208](https://github.com/ordinals/ord/pull/208) by [casey](https://github.com/casey))
- Re-implement find ([#206](https://github.com/ordinals/ord/pull/206) by [terror](https://github.com/terror))
- Add explanation to bounty page ([#205](https://github.com/ordinals/ord/pull/205) by [casey](https://github.com/casey))
- Change bounty dir to bounties ([#204](https://github.com/ordinals/ord/pull/204) by [casey](https://github.com/casey))
- Add ordinal bounty page ([#203](https://github.com/ordinals/ord/pull/203) by [terror](https://github.com/terror))
- Add drawbacks section to BIP ([#202](https://github.com/ordinals/ord/pull/202) by [casey](https://github.com/casey))
- Remove log spam ([#200](https://github.com/ordinals/ord/pull/200) by [casey](https://github.com/casey))
- Don't reopen LMDB databases ([#201](https://github.com/ordinals/ord/pull/201) by [casey](https://github.com/casey))
- Add serve recipe ([#199](https://github.com/ordinals/ord/pull/199) by [casey](https://github.com/casey))
- Continuously index ranges ([#198](https://github.com/ordinals/ord/pull/198) by [terror](https://github.com/terror))
- Add about page to website ([#197](https://github.com/ordinals/ord/pull/197) by [casey](https://github.com/casey))
- Put script tag in <head> ([#195](https://github.com/ordinals/ord/pull/195) by [casey](https://github.com/casey))
- Add list form ([#194](https://github.com/ordinals/ord/pull/194) by [terror](https://github.com/terror))
- Run server command ([#193](https://github.com/ordinals/ord/pull/193) by [casey](https://github.com/casey))
- Remove find command and KEY_TO_SATPOINT table ([#192](https://github.com/ordinals/ord/pull/192) by [casey](https://github.com/casey))
- Make checkout script check out correct branch ([#191](https://github.com/ordinals/ord/pull/191) by [casey](https://github.com/casey))
- Add server subcommand ([#185](https://github.com/ordinals/ord/pull/185) by [terror](https://github.com/terror))
- Use anyhow to add context to error messages ([#184](https://github.com/ordinals/ord/pull/184) by [casey](https://github.com/casey))
- Automate deployment ([#187](https://github.com/ordinals/ord/pull/187) by [casey](https://github.com/casey))
- Add ordinals.com website source ([#186](https://github.com/ordinals/ord/pull/186) by [casey](https://github.com/casey))
- Add LMDB database backend ([#177](https://github.com/ordinals/ord/pull/177) by [casey](https://github.com/casey))
- Link to project board in readme ([#176](https://github.com/ordinals/ord/pull/176) by [casey](https://github.com/casey))
- Test null outputs and inputs ([#169](https://github.com/ordinals/ord/pull/169) by [casey](https://github.com/casey))
- Log transaction indexing ([#168](https://github.com/ordinals/ord/pull/168) by [casey](https://github.com/casey))
- Remove the acknowledgements section since it's still a draft ([#164](https://github.com/ordinals/ord/pull/164) by [casey](https://github.com/casey))
- Add index size to info subcommand ([#162](https://github.com/ordinals/ord/pull/162) by [terror](https://github.com/terror))
- Document duplicate txid behavior ([#161](https://github.com/ordinals/ord/pull/161) by [casey](https://github.com/casey))
- Update redb 0.0.5 ([#160](https://github.com/ordinals/ord/pull/160) by [cberner](https://github.com/cberner))
- Document terminology and notation ([#158](https://github.com/ordinals/ord/pull/158) by [casey](https://github.com/casey))
- Describe dust output avoidance workaround ([#156](https://github.com/ordinals/ord/pull/156) by [casey](https://github.com/casey))
- Improve readme ([#154](https://github.com/ordinals/ord/pull/154) by [casey](https://github.com/casey))
- Improve find height check ([#150](https://github.com/ordinals/ord/pull/150) by [casey](https://github.com/casey))
- Use index for find queries ([#149](https://github.com/ordinals/ord/pull/149) by [casey](https://github.com/casey))
- Note that LN cannot be used to transfer individual ordinals ([#147](https://github.com/ordinals/ord/pull/147) by [casey](https://github.com/casey))
- Print block transaction count ([#146](https://github.com/ordinals/ord/pull/146) by [casey](https://github.com/casey))
- Use clap instead of structopt ([#145](https://github.com/ordinals/ord/pull/145) by [casey](https://github.com/casey))
- Incremental indexing ([#141](https://github.com/ordinals/ord/pull/141) by [casey](https://github.com/casey))
- Use human readable byte values for info ([#144](https://github.com/ordinals/ord/pull/144) by [casey](https://github.com/casey))
- Add info subcommand ([#138](https://github.com/ordinals/ord/pull/138) by [casey](https://github.com/casey))
- Accept human readable --index-size values ([#142](https://github.com/ordinals/ord/pull/142) by [casey](https://github.com/casey))
- Use redb::TableDefinition ([#143](https://github.com/ordinals/ord/pull/143) by [casey](https://github.com/casey))
- Work with live Bitcoin Core RPC API ([#140](https://github.com/ordinals/ord/pull/140) by [casey](https://github.com/casey))
- Use JSON RPC API  instead of blocksdir([#139](https://github.com/ordinals/ord/pull/139) by [casey](https://github.com/casey))
- Test mining and spending transactions in the same block ([#136](https://github.com/ordinals/ord/pull/136) by [terror](https://github.com/terror))
- Don't recreate db every run ([#131](https://github.com/ordinals/ord/pull/131) by [terror](https://github.com/terror))
- Fix off by one error in log message ([#135](https://github.com/ordinals/ord/pull/135) by [casey](https://github.com/casey))
- Improve index performance ([#134](https://github.com/ordinals/ord/pull/134) by [casey](https://github.com/casey))
- Reference independent invention ([#133](https://github.com/ordinals/ord/pull/133) by [casey](https://github.com/casey))
- Decode block header only in Index::index_blockfiles ([#132](https://github.com/ordinals/ord/pull/132) by [casey](https://github.com/casey))
- Add index benchmark ([#111](https://github.com/ordinals/ord/pull/111) by [casey](https://github.com/casey))
- Mention physical transfer of ordinals ([#130](https://github.com/ordinals/ord/pull/130) by [casey](https://github.com/casey))
- Reorder BIP sections ([#129](https://github.com/ordinals/ord/pull/129) by [casey](https://github.com/casey))
- Add applications section to BIP ([#127](https://github.com/ordinals/ord/pull/127) by [casey](https://github.com/casey))
- Add initial draft of BIP ([#117](https://github.com/ordinals/ord/pull/117) by [casey](https://github.com/casey))
- Test that index handles out-of-order blockfiles ([#124](https://github.com/ordinals/ord/pull/124) by [casey](https://github.com/casey))
- Test fee assignment ([#122](https://github.com/ordinals/ord/pull/122) by [terror](https://github.com/terror))
- Test underpaying subsidy ([#121](https://github.com/ordinals/ord/pull/121) by [terror](https://github.com/terror))
- Allow setting index size ([#120](https://github.com/ordinals/ord/pull/120) by [terror](https://github.com/terror))
- Use redb 0.0.4 ([#114](https://github.com/ordinals/ord/pull/114) by [casey](https://github.com/casey))
- Add duplicate transaction range test ([#113](https://github.com/ordinals/ord/pull/113) by [terror](https://github.com/terror))
- Split up Index::index_blockfiles ([#96](https://github.com/ordinals/ord/pull/96) by [casey](https://github.com/casey))
- Allow invalid ordinals ([#95](https://github.com/ordinals/ord/pull/95) by [casey](https://github.com/casey))
- Don't hardcode genesis block ([#91](https://github.com/ordinals/ord/pull/91) by [casey](https://github.com/casey))
- Rename index_blockfile to index_blockfiles ([#90](https://github.com/ordinals/ord/pull/90) by [casey](https://github.com/casey))
- Pin redb to GitHub revision to avoid panic ([#89](https://github.com/ordinals/ord/pull/89) by [casey](https://github.com/casey))
- Log progress while indexing ([#88](https://github.com/ordinals/ord/pull/88) by [casey](https://github.com/casey))
- Index all files in blocksdir ([#87](https://github.com/ordinals/ord/pull/87) by [casey](https://github.com/casey))
- Fix crash when indexing a block with no transactions ([#86](https://github.com/ordinals/ord/pull/86) by [casey](https://github.com/casey))
- Refactor test API ([#82](https://github.com/ordinals/ord/pull/82) by [terror](https://github.com/terror))
- More integration test cleanup ([#70](https://github.com/ordinals/ord/pull/70) by [casey](https://github.com/casey))
- Refactor test block creation ([#68](https://github.com/ordinals/ord/pull/68) by [terror](https://github.com/terror))
- Improve index ([#60](https://github.com/ordinals/ord/pull/60) by [casey](https://github.com/casey))
- Add `index.redb` to gitignore ([#58](https://github.com/ordinals/ord/pull/58) by [casey](https://github.com/casey))
- Make find command print satpoints instead of outpoints ([#57](https://github.com/ordinals/ord/pull/57) by [casey](https://github.com/casey))
- Improve transfer algorithm pseudocode ([#53](https://github.com/ordinals/ord/pull/53) by [casey](https://github.com/casey))
- Add epoch trait ([#51](https://github.com/ordinals/ord/pull/51) by [casey](https://github.com/casey))
- Use strong types ([#48](https://github.com/ordinals/ord/pull/48) by [casey](https://github.com/casey))
- Add Index struct ([#47](https://github.com/ordinals/ord/pull/47) by [casey](https://github.com/casey))
- Use ordinal number terminology ([#46](https://github.com/ordinals/ord/pull/46) by [casey](https://github.com/casey))
- Number satoshis in ascending order ([#45](https://github.com/ordinals/ord/pull/45) by [casey](https://github.com/casey))
- Use default location if `--blocksdir` is not provided ([#42](https://github.com/ordinals/ord/pull/42) by [casey](https://github.com/casey))
- Update dependencies ([#40](https://github.com/ordinals/ord/pull/40) by [casey](https://github.com/casey))
- Create illusive and cursed traits ([#36](https://github.com/ordinals/ord/pull/36) by [casey](https://github.com/casey))
- Add character trait ([#35](https://github.com/ordinals/ord/pull/35) by [casey](https://github.com/casey))
- Add open questions to readme ([#34](https://github.com/ordinals/ord/pull/34) by [casey](https://github.com/casey))
- Use descending numbering scheme ([#33](https://github.com/ordinals/ord/pull/33) by [casey](https://github.com/casey))
- Handle out-of-bound values ([#30](https://github.com/ordinals/ord/pull/30) by [casey](https://github.com/casey))
- Add yet more traits ([#29](https://github.com/ordinals/ord/pull/29) by [casey](https://github.com/casey))
- Add shiny trait ([#28](https://github.com/ordinals/ord/pull/28) by [casey](https://github.com/casey))
- Add command to find satoshi with a given name ([#27](https://github.com/ordinals/ord/pull/27) by [casey](https://github.com/casey))
- Add more traits ([#25](https://github.com/ordinals/ord/pull/25) by [casey](https://github.com/casey))
- Add traits ([#24](https://github.com/ordinals/ord/pull/24) by [casey](https://github.com/casey))
- Add readme and refactor code ([#22](https://github.com/ordinals/ord/pull/22) by [casey](https://github.com/casey))
- Rename to sat-tracker ([#21](https://github.com/ordinals/ord/pull/21) by [casey](https://github.com/casey))
- Start new sat-based implementation ([#20](https://github.com/ordinals/ord/pull/20) by [casey](https://github.com/casey))
- Add justfile and catalog recipe ([#12](https://github.com/ordinals/ord/pull/12) by [casey](https://github.com/casey))
- Organize code ([#10](https://github.com/ordinals/ord/pull/10) by [casey](https://github.com/casey))
- Add supply command ([#9](https://github.com/ordinals/ord/pull/9) by [casey](https://github.com/casey))
- Track atom locations ([#2](https://github.com/ordinals/ord/pull/2) by [casey](https://github.com/casey))
- Add Rust binary and CI workflow ([#1](https://github.com/ordinals/ord/pull/1) by [casey](https://github.com/casey))
- Add readme
