# リリース形式
KomeおよびmochiOS ToolchainのリリースはGitHub Releasesから配布します。

## フォーマット
リリースアーカイブには`tar.zst`を使用します。

ファイル名は次の形式です: `kome-{architecture}-{platform}.tar.zst`

- architecture
  - `x86_64`
  - `mrv64`
  のいずれかをとります

- platform
  - `linux`
  - `windows`
  - `mochios`
  のいずれかをとります

## ファイル構造
アーカイブのルートは、そのままKome のインストールディレクトリへ展開できる構造にします。
必要なのは次です。

- bin/
- lib/
- stdlib/
- sdk/

### bin

実行可能ファイルを格納します。
最低限、以下を含んでください。

- bin/kome
- bin/komec
- bin/x86_64-mochios-clang
- bin/x86_64-mochios-ld

アーキテクチャ固有ツールは、対象アーキテクチャに応じた名前を使用してください。

### lib

Kome runtimeなど、Komeのコンパイル・実行に必要なライブラリを格納します。

### stdlib

Kome標準ライブラリを格納します。
コンパイラは通常、このディレクトリをインストール先から自動検出します。

### sdk

mochiOS向けクロスコンパイルに必要なSDKを格納します。

最低限、以下を含んでください。

- sdk/lib/crt0.o
- sdk/lib/linker.ld
- sdk/lib/libmochi_user_newlib_runtime.a
- sdk/lib/libgcc.a
- sdk/sysroot/

`sysroot`にはnewlibのヘッダ、`libc.a`、`libm.a`などを含めます。

## インストール先

`komeup`の標準インストール先は次の場所です: `~/.kome/`
アーカイブはこのディレクトリへ直接展開されます。

環境変数 `KOME_HOME` が設定されている場合は、そのディレクトリを代わりに使用します。

## GitHub Release assets

1つのGitHub Releaseには、対応する各ホスト環境向けのアーカイブを添付します。

例:

```text
kome-x86_64-linux.tar.zst
kome-aarch64-linux.tar.zst
SHA256SUMS
```

## Checksums

各Releaseには`SHA256SUMS`を添付します。

形式:
```text
<sha256>  kome-x86_64-linux.tar.zst
<sha256>  kome-aarch64-linux.tar.zst
```

`komeup`はアーカイブを展開する前にchecksumを検証します。

checksumが一致しない場合、そのリリースをインストールしてはいけません。

## Compatibility

リリースアーカイブ内の以下の要素は、同一リリースとして互換性のある組み合わせにします。

- `kome`
- `komec`
- Kome runtime
- stdlib
- mochiOS Clang driver
- mochiOS linker driver
- mochiOS SDK

異なるリリースから個別にファイルを混在させることは保証しません。

## Release creation

Release作成時は、インストール先と同じディレクトリ構造を作成してからアーカイブします。

例:

```bash
tar -C release-root -cf - . | zstd -19 -o kome-x86_64-linux.tar.zst
```

その後SHA-256を生成します。

```bash
sha256sum kome-*.tar.zst > SHA256SUMS
```
