# SimpleBTC Documentation

SimpleBTC项目的完整文档站点。

## 本地预览

### 使用mdBook（推荐）

1. 安装mdBook：

```bash
cargo install mdbook
```

2. 构建并预览：

```bash
cd docs
mdbook serve --open
```

文档将在 `http://localhost:3000` 打开。

### 仅构建

```bash
cd docs
mdbook build
```

生成的HTML文件在 `docs/book/` 目录。

## 部署到GitHub Pages

### 方法1: 手动部署

```bash
# 构建文档
cd docs
mdbook build

# 复制到gh-pages分支
git checkout --orphan gh-pages
git rm -rf .
cp -r docs/book/* .
git add .
git commit -m "Update documentation"
git push origin gh-pages --force
```

### 方法2: GitHub Actions（推荐）

在 `.github/workflows/docs.yml` 添加：

```yaml
name: Deploy Documentation

on:
  push:
    branches: [ main ]
  pull_request:
    branches: [ main ]

jobs:
  deploy:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v3

      - name: Install mdBook
        run: |
          cargo install mdbook

      - name: Build Documentation
        run: |
          cd docs
          mdbook build

      - name: Deploy to GitHub Pages
        if: github.ref == 'refs/heads/main'
        uses: peaceiris/actions-gh-pages@v3
        with:
          github_token: ${{ secrets.GITHUB_TOKEN }}
          publish_dir: ./docs/book
```

访问：`https://yourusername.github.io/SimpleBTC/`

## 独立部署

### 使用Nginx

```nginx
server {
    listen 80;
    server_name docs.simplebtc.com;

    root /var/www/simplebtc-docs/book;
    index index.html;

    location / {
        try_files $uri $uri/ =404;
    }
}
```

```bash
# 部署步骤
mdbook build
sudo cp -r book/* /var/www/simplebtc-docs/book/
sudo systemctl reload nginx
```

### 使用Docker

创建 `Dockerfile`:

```dockerfile
FROM nginx:alpine
COPY docs/book /usr/share/nginx/html
EXPOSE 80
CMD ["nginx", "-g", "daemon off;"]
```

构建和运行：

```bash
docker build -t simplebtc-docs .
docker run -d -p 8080:80 simplebtc-docs
```

访问 `http://localhost:8080`

## 文档结构

```
docs/
├── book.toml              # mdBook配置
├── src/                   # 源文件
│   ├── SUMMARY.md        # 目录结构
│   ├── introduction/     # 介绍
│   ├── guide/            # 使用指南
│   ├── examples/         # 实战案例
│   ├── api/              # API参考
│   ├── advanced/         # 高级特性
│   └── appendix/         # 附录
└── book/                 # 生成的HTML（git忽略）
```

## 贡献文档

### 添加新页面

1. 在 `src/` 下创建 `.md` 文件
2. 在 `SUMMARY.md` 中添加链接
3. 使用mdBook预览
4. 提交PR

### Markdown格式规范

```markdown
# 一级标题（每页只有一个）

## 二级标题

### 三级标题

**粗体** *斜体* `代码`

\```rust
// 代码块
fn main() {
    println!("Hello");
}
\```

> 引用块

| 列1 | 列2 |
|-----|-----|
| A   | B   |

- 列表项
  - 子项

[链接](./path/to/page.md)
```

### 中英双语支持

目前文档为中文。如需添加英文版：

1. 创建 `docs-en/` 目录
2. 复制结构并翻译
3. 修改 `book.toml`:

```toml
[book]
multilingual = true
language = "zh"

[translations.en]
title = "SimpleBTC Documentation (English)"
```

4. 添加语言切换器

## 自动化构建

### 监听文件变化

```bash
# mdBook自动重新构建
mdbook watch
```

### Pre-commit Hook

在 `.git/hooks/pre-commit` 添加：

```bash
#!/bin/bash
cd docs
mdbook test
if [ $? -ne 0 ]; then
    echo "Documentation build failed"
    exit 1
fi
```

## 文档维护

### 检查链接

```bash
# 安装链接检查器
cargo install mdbook-linkcheck

# 在 book.toml 添加
[output.linkcheck]

# 构建时自动检查
mdbook build
```

### 更新目录

编辑 `src/SUMMARY.md`：

```markdown
# Summary

[介绍](./introduction/README.md)

# 快速开始
- [安装](./guide/installation.md)
- [教程](./guide/quickstart.md)

...
```

## 常见问题

### mdBook未安装

```bash
cargo install mdbook
```

### 构建失败

```bash
# 清理并重建
mdbook clean
mdbook build
```

### 端口占用

```bash
# 使用不同端口
mdbook serve --port 3001
```

### 样式自定义

在 `book.toml` 添加：

```toml
[output.html]
additional-css = ["custom.css"]
additional-js = ["custom.js"]
```

## 资源

- [mdBook文档](https://rust-lang.github.io/mdBook/)
- [Markdown指南](https://www.markdownguide.org/)
- [GitHub Pages](https://pages.github.com/)

---

Happy documenting! 📚
