江海浮天一叶舟，久困囚居终有时。初入 Rust Web，可能需适应，但熟练后如鱼得水，必将安然入眠，性能可靠，亦少错漏。
祝君不负韶华，道阻且长，行则将至。

# 介绍

这是一个由 [salvo-cli](https://github.com/salvo-rs/salvo-cli) 生成的项目，你可以按照以下命令来运行程序以及测试 (非 sqlite 数据库的请先按照教程修改数据库连接串，完成数据的初始工作)。
😄 最新版的 Salvo 依赖 Rust 版本 1.80。如果编译失败，请尝试使用 `rustup update` 来升级版本。

``` shell
//运行项目
cargo run
//运行测试
cargo test
```

# 小贴士

- 如果数据库是 sqlite 或已经运行了数据库迁移，请使用账号 zhangsan 密码 123 来登录系统。
- 程序数据库连接串在 config/config.toml 里，但是如果你使用的是 sqlx 或者 seaorm，库本身读取 .env 文件的配置来生成实体，运行迁移，验证。所以当你修改数据库连接串时，需要同时修改两个地方。

# orm 的文档或主页链接

🎯 您选择了 sqlx，文档可以在这里查看:<https://github.com/launchbadge/sqlx>

## sqlx_cli

SQLx 的相关命令行实用程序，用于管理数据库、迁移以及使用 sqlx::query!() 等启用“脱机”模式。 <https://github.com/launchbadge/sqlx/blob/main/sqlx-cli/README.md>

## 数据初始化

你选择的是 sqlite 数据库，数据库已初始化完毕，在 data 文件夹下。

# 关于赛风 (salvo)

你可以在 <https://salvo.rs/> 📖查看 salvo 的文档以及更多例子，如果我们的工具帮到你，欢迎 star [salvo](https://github.com/salvo-rs/salvo) 和 [salvo-cli](https://github.com/salvo-rs/salvo-cli),这将给我们很大激励。❤️️

# Install

## Windows

install openssl:x64-windows-static-md

```shell
vcpkg install openssl:x64-windows-static-md
```
