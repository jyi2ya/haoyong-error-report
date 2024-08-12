# haoyong-error-report

导引 [miette](https://github.com/zkat/miette) 和 [poem-openapi](https://github.com/poem-web/poem) 之力缝合的好用错误汇报库。让错误可以方便地格式化并在摘要、日志、命令行和 RESTful API 中使用。😋😋

## 使用教学

首先 cargo add 这个库。因为暂时没放到 crates.io 上，所以需要直接 add git 仓库。

然后定义错误（这里用的是 snafu，用 thiserror 之类的也是可以的）：

```rust
use miette::Diagnostic;

#[derive(Debug, Snafu, Diagnostic)]
#[diagnostic(url("https://about.villv.tech/"))]
pub enum ApiDbError {
    #[snafu(display("failed to connect to api db: mysql://{user}@{host}:{port}/{name}"))]
    #[diagnostic(code(E114514), help("连不上数据库，也许是配置或网络出了问题"))]
    ConnectionFailed {
        #[snafu(source)]
        source: DbErr,

        user: String,
        host: Ipv4Addr,
        port: u16,
        name: String,
    },
}
```

现在已经万事具备了，倘若哪里出了错，我们可以选择喜欢的方式来汇报错误。

---

摘要的版本：

```rust
api_db_err.to_brief_report()
```

得到：

```plain
failed to connect to api db: mysql://xxx@x.y.z/ww
```

---

输出单行，用来打日志的版本：

```rust
api_db_err.to_detailed_singleline_report()
```

得到：

```plain
failed to connect to api db: mysql://xxx@x.y.z/ww | Connection Error: error communicating with database: expected to read 4 bytes, got 0 bytes at EOF <<< error communicating with database: expected to read 4 bytes, got 0 bytes at EOF <<< error communicating with database: expected to read 4 bytes, got 0 bytes at EOF <<< expected to read 4 bytes, got 0 bytes at EOF
```

---

输出多行，我也不知道用来干啥的版本：

```rust
api_db_err.to_detailed_multiline_report()
```

得到：

```plain
failed to connect to api db: mysql://mysql://xxx@x.y.z/ww

Caused By:
    [1] Connection Error: error communicating with database: expected to read 4 bytes, got 0 bytes at EOF
    [2] error communicating with database: expected to read 4 bytes, got 0 bytes at EOF
    [3] error communicating with database: expected to read 4 bytes, got 0 bytes at EOF
    [4] expected to read 4 bytes, got 0 bytes at EOF
```

---

使用 miette 来生成漂亮的终端输出的版本：

```rust
api_db_err.into_fancy_cli_report()
```

得到：

```plain
E114514 (link)

  × failed to connect to api db: mysql://mysql://xxx@x.y.z/ww
  ├─▶ Connection Error: error communicating with database: expected to read 4 bytes, got 0 bytes at EOF
  ├─▶ error communicating with database: expected to read 4 bytes, got 0 bytes at EOF
  ├─▶ error communicating with database: expected to read 4 bytes, got 0 bytes at EOF
  ╰─▶ expected to read 4 bytes, got 0 bytes at EOF
  help: 连不上数据库，也许是配置或网络出了问题
```

---

此外，还能直接和 poem 连携使用。

编写：

```rust
struct play;

#[derive(ApiResponse)]
enum IndexResponseError {
    #[oai(status = 500)]
    DbError(HaoyongPoemResponse),
}

#[poem_openapi::OpenApi]
impl play {
    #[oai(path = "/", method = "get")]
    async fn index(&self, PlainText(req): PlainText<String>) -> Result<(), IndexResponseError> {
        let config = config::Config::try_load(::config::File::with_name("config")).unwrap();
        client::ApiDb::try_connect(&config)
            .await
            .map_err_to_poem_response()
            .map_err(IndexResponseError::DbError)?;
        Ok(())
    }
}
```

访问：

```sh
curl -H 'Content-Type: text/plain' localhost:3000/api | jq
```

得到：

```plain
{
  "code": "E114514",
  "detail": "failed to connect to api db: mysql://mysql://xxx@x.y.z/ww | Connection Error: error communicating with database: expected to read 4 bytes, got 0 bytes at EOF <<< error communicating with database: expected to read 4 bytes, got 0 bytes at EOF <<< error communicating with database: expected to read 4 bytes, got 0 bytes at EOF <<< expected to read 4 bytes, got 0 bytes at EOF",
  "help": "连不上数据库，也许是配置或网络出了问题",
  "message": "failed to connect to api db: mysql://mysql://xxx@x.y.z/ww",
  "doc": "https://about.villv.tech/"
}
```

好用捏