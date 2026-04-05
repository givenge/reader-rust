# 本地开发
1. 安装依赖
   ```bash
   npm install
   ```

2. 启动开发服务器
   ```bash
   npm run docs:dev
   ```

3. 打开浏览器访问 `http://localhost:5173`

# 构建

```bash
npm run docs:build
```

构建输出在 `.vitepress/dist` 目录。

# 预览

```bash
npm run docs:preview
```

# 部署到 GitHub Pages

## 方法一：GitHub Actions (推荐)

1. 在项目根目录创建 `.github/workflows/docs.yml`:

```yaml
name: Deploy Docs

on:
  push:
    branches: [ master ]

jobs:
  deploy:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      - uses: actions/setup-node@v4
        with:
          node-version: 20
      - run: cd docs && npm ci
      - run: cd docs && npm run docs:build
      - name: Deploy
        uses: peaceiris/actions-gh-pages@v3
        with:
          github_token: ${{ secrets.GITHUB_TOKEN }}
          publish_dir: ./docs/.vitepress/dist
```

2. 在 GitHub 仓库设置中启用 Pages，选择 `gh-pages` 分支

## 方法二：手动部署

1. 进入 docs 目录
   ```bash
   cd docs
   ```

2. 构建并初始化 git
   ```bash
   npm run docs:build
   cd .vitepress/dist
   git init
   git add -A
   git commit -m 'deploy'
   ```

3. 推送到 gh-pages 分支
   ```bash
   git push -f https://github.com/givenge/reader-rust.git master:gh-pages
   ```

4. 访问 `https://givenge.github.io/reader-rust`

# 自定义域名 (可选)

在 `docs/.vitepress/public/` 目录下创建 `CNAME` 文件：

```
your-domain.com
```
