# Lagosta 🦞
<img width="128" height="128" alt="Lagosta" src="assets/lagosta.png" />

[Ícone tirado do emoji kitchen da google](https://emojikitchen.dev/)

# Compilação
Requer `cargo`, `godot` e, opcionalmente, `docker`

## 1. Biblioteca

> _a lagosta veio do lago_

O projeto é composto de duas partes:
- Uma biblioteca GDExtension, escrita em Rust - `Lago`
- O projeto geral, feito em Godot - `Lagosta`

A gente usa um script auxiliar de build pra compilar a biblioteca.
Ele detecta se o sistema tem o docker instalado e usa ele pra compilar uma versão cross-platform.
(Ele baseia a biblioteca numa `glibc` mais antiga pra garantir compatibilidade com mais sistemas)

Se o sistema não tem docker instalado, ele builda nativamente mesmo.

#### ⚠️ Por favor, não builde uma release final do aplicativo _nativamente_ (sem o docker)

Para compilar a biblioteca basta rodar `rust/task_build.sh`
```bash
cd rust
# ./task_build.sh flags:
#         --win (builda pra windows),
#         --all (builda pra todas as plataformas),
#         --release (builda a versão release otimizada da biblioteca)
./task_build.sh
```

## 2. App Lagosta
Com o projeto aberto dentro da Godot, vá em `Project > Export` e exporte a versão desejada na pasta `bin/`.

**Não esqueça de desativar o export com debug para utilizar a versão release da biblioteca.**

<img width="244" height="118" alt="image" src="https://github.com/user-attachments/assets/2446b915-91f1-4c21-a561-328d124451ac" />
<img width="646" height="292" alt="image" src="https://github.com/user-attachments/assets/c8730373-0d33-4734-b979-16bf60160eee" />
