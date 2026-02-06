# kakidashi

[青空文庫](https://www.aozora.gr.jp/index.html)に収蔵された作品の書き出し1文を出力するCLIツールです。

- 作品テキストは[青空文庫GitHubリポジトリ](https://github.com/aozorabunko/aozorabunko)から取得
- 著作権が消滅した作品のみを使用
- ただし全作品が出力されるとは限らない (青空文庫形式テキストから正しく書き出し1文を抽出できていない作品多数)

## インストール

### Linux (x86_64)

```bash
curl -fsSL https://github.com/ciray/kakidashi/releases/latest/download/kakidashi-x86_64-unknown-linux-gnu.tar.gz | sudo tar -xzC /usr/local/bin
```

## 機能

```bash
$ kakidashi --help
Display the opening sentence of works from Aozora Bunko

Usage: kakidashi [OPTIONS]

Options:
  -n, --number <NUMBER>      Number to output [default: 1]
  -a, --all                  Output all [conflicts with --number]
      --no-random            Disable randomization
  -q, --query <QUERY>        Filter queries [format: key=value] [possible keys: author, title, text]
  -i, --interactive          Interactive selection mode [conflicts with --query]
  -f, --format <FORMAT>      Output format [default: plain] [possible values: plain, quote, csv, json]
  -t, --template <TEMPLATE>  Template only for 'quote' format [possible laceholders: {author}, {title}, {text}, {url}. example: '{text} - {author} ({title})']
  -h, --help                 Print help
  -V, --version              Print version
```

## 使用例

### 1件をランダム出力

```bash
$ kakidashi
親譲りの無鉄砲で小供の時から損ばかりして居る。
```

### 全件出力

```bash
$ kakidashi --all | wc -l
14962
```

### 作家/作品でフィルタリング

```bash
$ kakidashi --query author="南方 熊楠" --query title="十二支考"
隙行く駒の足早くて午の歳を迎うる今日明日となった。
```

### 作家/作品を対話的に選択

```bash
$ kakidashi --interactive
> Select author: 中原 中也
> Select title: 作家と孤独
インテリは蒼ざめてゐる。
```

### 出力テンプレートを指定 (ついでにcowsay)

```bash
$ kakidashi --format quote --template "{text}\n    {author}  『{title}』" | cowsay
 ________________________________
/ よだかは、実にみにくい鳥です。 \
|                                |
\ 宮沢 賢治 『よだかの星』       /
 --------------------------------
        \   ^__^
         \  (oo)\_______
            (__)\       )\/\
                ||----w |
                ||     ||
```

### 作品ページをブラウザで表示 (Windows/WSL + Chrome例)

```bash
$ kakidashi --format json | jq .url | xargs chrome.exe
```
