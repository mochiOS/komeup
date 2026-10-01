# リリース形式

Kome と mochiOS Toolchain は GitHub Releases から配布します。

## 配布元

`komeup install` は以下のReleaseを使用します。

- `mochiOS/komec`
- `mochiOS/komeup`
- `mochiOS/toolchains`
- `mochiOS/devkit`

## Kome

`mochiOS/komec` は Kome 本体、Komeコンパイラ、runtime、標準ライブラリを配布します。

```text
{arch}-kome-{platform}.tar.zst
{arch}-kome-std-{platform}.tar.zst
SHA256SUMS
```

`{arch}-kome-{platform}.tar.zst` には `kome`、`komec` と必要なruntimeを含めます。

標準ライブラリは `{arch}-kome-std-{platform}.tar.zst` として分離します。

## komeup

`mochiOS/komeup` は komeup 自身を配布します。

```text
{arch}-komeup-{platform}.tar.zst
SHA256SUMS
```

## mochiOS Toolchain

`mochiOS/toolchains` は mochiOS 向けクロスツールチェーンとSDKを配布します。

アーカイブには最低限以下を含めます。

- `bin/{arch}-mochios-clang`
- `bin/{arch}-mochios-ld`
- `sdk/lib/crt0.o`
- `sdk/lib/linker.ld`
- `sdk/lib/libmochi_user_newlib_runtime.a`
- `sdk/lib/libgcc.a`
- `sdk/sysroot/`

## AppCore

`mochiOS/devkit` は AppCore を配布します。

```text
{arch}-appcore-{version}.zst
SHA256SUMS
```

`version` は GitHub Release のタグから先頭の `v` を除いた値を使用します。

AppCore は `~/.kome/appcore/`、または `KOME_HOME/appcore/` に展開します。

## インストール先

標準のインストール先は `~/.kome/` です。

`KOME_HOME` が設定されている場合は、そのディレクトリを使用します。

各アーカイブは `KOME_HOME` へ直接展開できる構造にします。

## Checksums

各Releaseには `SHA256SUMS` を必ず添付します。

```text
<sha256>  <archive name>
```

`komeup` はすべてのアーカイブを展開前にSHA-256で検証します。

checksumが一致しない、または対象ファイルが `SHA256SUMS` に存在しない場合はインストールを中止します。
