# make_wallpaper

画像をMac用の壁紙に変換するCLIツール。

## 機能

- 画像の中でMacのメニューバーが表示される部分を黒く潰す
- 指定した解像度にリサイズする
  - 画像の方が小さい場合は拡大する
  - 画像の方が大きい場合は圧縮し、アスペクト比が異なる場合は中央を固定してトリミングする
- 画像全体をぼかす（オプション）
- 角を丸くする（オプション）

## スタイルテンプレート

解像度やメニューバーの高さなどの設定をTOMLファイルにテンプレートとして保存し、コマンド実行時に呼び出すことができる。

### 設定ファイルの検索順序

1. `-c, --config` オプションで指定されたパス
2. `$XDG_CONFIG_HOME/make_wallpaper/styles.toml`
3. `~/.config/make_wallpaper/styles.toml`
4. `~/.make_wallpaper/styles.toml`

### 設定ファイルの例

```toml
# MacBook Air 15" (2880x1864)
[macbook_air_15]
width = 1710
height = 1107
menubar_height = 37

# WQHD Monitor (2560x1440)
[wqhd_monitor]
width = 2560
height = 1440
menubar_height = 24
```

## 使い方

```
make_wallpaper <input_image> --style <style_name> [options]
```

### 引数

| オプション | 短縮形 | 説明 |
|-----------|--------|------|
| `<input_image>` | - | 入力画像のパス（JPEG、PNG対応） |
| `--style <name>` | `-t` | スタイルテンプレート名（必須） |
| `--config <path>` | `-c` | 設定ファイルのパス |
| `--size <WxH>` | `-s` | 解像度を上書き（例: `1920x1080`） |
| `--menubar-height <px>` | `-m` | メニューバーの高さを上書き |
| `--blur [radius]` | `-b` | ぼかし半径を上書き（デフォルト: 10、`0`で無効化） |
| `--round-corners [radius]` | `-r` | 角丸半径を上書き（デフォルト: 20） |
| `--out-dir <dir>` | `-o` | 出力ディレクトリ（デフォルト: カレント） |
| `--help` | `-h` | ヘルプを表示 |

### 出力ファイル名

出力ファイル名は以下の形式になる:

- 上書きオプションなし: `<元のファイル名>_<スタイル名>.png`
- 上書きオプションあり: `<元のファイル名>_<スタイル名>_<変更パラメータ>.png`

変更パラメータの略称:
- サイズ: `1920x1080`
- メニューバー高さ: `m24`
- ぼかし: `b10`
- 角丸: `r20`

### 使用例

```bash
# 基本的な使い方
make_wallpaper image.jpg -t macbook_air_15

# 出力ディレクトリを指定
make_wallpaper image.jpg -t macbook_air_15 -o ./output

# スタイルの一部を上書き
make_wallpaper image.jpg -t wqhd_monitor --blur 15 --round-corners 30

# styles.tomlでblurが設定されていても無効化
make_wallpaper image.jpg -t wqhd_monitor --blur 0

# 設定ファイルを指定
make_wallpaper image.jpg -t my_style -c /path/to/my_styles.toml

# 複数のオプションを上書き
make_wallpaper image.jpg -t macbook_air_15 -s 1920x1080 -m 30 -b -r
# 出力: image_macbook_air_15_1920x1080_m30_b10_r20.png
```

## ビルド

```bash
cargo build --release
```

## インストール

```bash
cargo install --path .
```
