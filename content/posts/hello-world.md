---
title: "Olá, mundo!"
created_at: 2026-03-31
summary: "Primeiro post do blog — por que decidi construir meu próprio site do zero com Rust, HTMX e zero JavaScript."
---

## Por que construir do zero?

Existem dezenas de geradores de sites estáticos por aí — Hugo, Zola, Astro, Next.js. Qualquer um deles resolveria o problema em uma tarde. Mas eu queria algo diferente.

Queria entender cada camada: como o servidor responde, como o HTML chega no navegador, como a navegação funciona sem recarregar a página. Não por necessidade, mas por curiosidade.

## A stack

O blog roda com:

- **Rust** com Axum como framework web
- **Askama** para templates compilados em tempo de build
- **HTMX** para navegação SPA-like sem escrever JavaScript
- **Markdown** com frontmatter YAML para os posts

Sem bundler. Sem node_modules. Sem framework de frontend. O servidor retorna HTML e o navegador renderiza. Simples assim.

## HATEOAS na prática

A arquitetura segue o princípio HATEOAS — o servidor é a única fonte de verdade. Cada resposta é HTML completo ou um fragmento. O cliente não gerencia estado, não faz fetch de JSON, não tem router próprio.

```rust
async fn blog_list(
    HxRequest(is_htmx): HxRequest,
) -> Result<impl IntoResponse, AppError> {
    let posts = load_all_posts("content/posts")?;
    if is_htmx {
        Ok(BlogListFragment { posts }.into_response())
    } else {
        Ok(BlogListPage { posts }.into_response())
    }
}
```

Quando o HTMX faz uma requisição, o handler retorna só o fragmento. Navegação direta retorna a página completa. Zero duplicação de lógica.

## Próximos passos

- Adicionar tags e filtro por tag
- Paginação com "carregar mais"
- Página de currículo interativa
- Talvez um tema claro (talvez não)

Por enquanto, o blog existe. E isso já é alguma coisa.
