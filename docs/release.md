# SDKリリース

Kome SDKは同じSDK版を持つ複数のGitHub Releaseから構成します。最初のSDK版は`27.0-dp.1`です。

## 配布元

- `mochiOS/komec`: `kome`、`komec`、`kome-lsp`、実行環境、標準ライブラリ
- `mochiOS/komeup`: インストーラー
- `mochiOS/toolchains`: mochiOS向けクロスツールチェーン
- `mochiOS/ViewKit`: ViewKitのKome APIと共有ライブラリ
- `mochiOS/devkit`: AppCoreのKome APIと共有・静的ライブラリ

すべてのリポジトリで同じタグ`27.0-dp.1`を使用します。`komeup`はlatestではなく、このタグを直接取得します。

## 成果物

`mochiOS/komec`:

```text
x86_64-kome-27.0-dp.1.tar.zst
x86_64-komec-27.0-dp.1.tar.zst
x86_64-kome-std-27.0-dp.1.tar.zst
SHA256SUMS
```

その他のリポジトリ:

```text
x86_64-komeup-27.0-dp.1.tar.zst
x86_64-mochios-toolchain-27.0-dp.1.tar.zst
x86_64-viewkit-27.0-dp.1.tar.zst
x86_64-appcore-27.0-dp.1.tar.zst
```

各Releaseには、そのRelease内の全アーカイブを列挙した`SHA256SUMS`を添付します。

## インストール配置

```text
~/.kome/
├── bin/
│   ├── kome
│   ├── komec
│   ├── kome-lsp
│   ├── komeup
│   └── libkome_native_rt.a
├── stdlib/
├── viewkit/
├── appcore/
├── sdk/
└── sdk-version
```

`komeup`は全成果物を取得して検証した後、一時ディレクトリへ展開し、SDK全体を一括で置き換えます。

## 作成手順

1. 各リポジトリの作業ツリーをクリーンにし、テストを完走させます。
2. Kome、ViewKit、AppCore、komeup、ツールチェーンの成果物を`27.0-dp.1`で生成します。
3. 各リポジトリに`27.0-dp.1`の下書きReleaseを作り、成果物と`SHA256SUMS`を添付します。
4. ViewKit、AppCore、ツールチェーン、Kome、komeupの順でReleaseを公開します。
5. 空の`KOME_HOME`を指定してインストール試験を行います。

```sh
KOME_HOME=/tmp/kome-sdk-test ./target/release/komeup install 27.0-dp.1
/tmp/kome-sdk-test/bin/kome check --manifest-path /path/to/project/Kome.toml
```

6. JIT実行、AOTビルド、生成バイナリ実行、`kome-lsp`起動を確認します。
7. インストール用スクリプトを確認します。

```sh
KOME_SDK_VERSION=27.0-dp.1 sh install.sh
```

## 更新と削除

```sh
komeup update 27.0-dp.1
komeup uninstall
```

版を省略した場合は、その`komeup`が既定として持つSDK版を使用します。
