---
title: "Olá, mundo!"
date: 2026-03-31
summary: "Primeiro post do blog — por que decidi construir meu próprio site do zero com Rust e zero JavaScript."
---

## Por que construir do zero?

Existem dezenas de geradores de sites estáticos por aí — Hugo, Zola, Astro, Next.js. Qualquer um deles resolveria o problema em uma tarde. Mas eu queria algo diferente.

Queria entender cada camada: como o servidor responde, como o HTML chega no navegador, como a navegação funciona sem JavaScript. Não por necessidade, mas por curiosidade.

## A stack

O blog roda com:

- **Rust** com Axum como framework web
- **Askama** para templates compilados em tempo de build
- **Markdown** com frontmatter YAML para os posts
- **syntect** para syntax highlighting feito no servidor

Sem bundler. Sem node_modules. Sem framework de frontend. Sem JavaScript. O servidor retorna HTML e o navegador renderiza. Simples assim.

## HATEOAS na prática

A arquitetura segue o princípio HATEOAS — o servidor é a única fonte de verdade. Cada resposta é uma página HTML completa. O cliente não gerencia estado, não faz fetch de JSON, não tem router próprio. Links e formulários são a API.

```rust
async fn blog_list(
    State(state): State<AppState>,
) -> Result<impl IntoResponse, AppError> {
    let store = state.posts()?;
    let html = BlogListTemplate {
        posts: store.all.clone(),
        layout: LayoutContext::new("/blog"),
    }
    .render()?;
    Ok(Html(html))
}
```

Um handler, um template, uma resposta. Toda a navegação é feita com `<a href>` comum — o navegador já sabe fazer isso há trinta anos.

## Próximos passos

- Adicionar tags e filtro por tag
- Paginação por links
- Página de currículo interativa
- Talvez um tema claro (talvez não)

Por enquanto, o blog existe. E isso já é alguma coisa.
