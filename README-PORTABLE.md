# Doop Portable V1

Esta branch transforma o desktop do Doop em uma distribuição portátil para Windows.

## Objetivo

Executar o Doop a partir de uma pasta, pendrive ou SSD externo sem exigir Docker, PostgreSQL, Node.js ou Bun instalados na máquina hospedeira.

A distribuição gerada pelo GitHub Actions contém:

- `DoopPortable.exe`: shell desktop Tauri.
- `runtime/bun.exe`: runtime que executa o servidor local.
- `app/`: frontend compilado, servidor, dependências e arquivos compartilhados.
- `data/`: dados persistentes do usuário.
- `config/auth.secret`: segredo local gerado automaticamente no primeiro uso.
- `logs/server.log`: log do servidor local.

## Fluxo de inicialização

1. `DoopPortable.exe` encontra sua própria pasta.
2. Cria `data/`, `config/` e `logs/` quando necessário.
3. Procura uma porta livre entre 4400 e 4449.
4. Inicia o servidor Doop em `127.0.0.1` usando o Bun incluído.
5. Define automaticamente a URL e o segredo do Better Auth.
6. O PGlite e os assets usam a pasta `data/` ao lado do executável.
7. O Tauri abre a interface local.
8. Ao fechar o aplicativo, o processo do servidor local é encerrado.

## Build

Use o workflow **Portable Windows** em Actions.

O artefato final é `DoopPortable-windows.zip`.

## Uso

Extraia o ZIP inteiro para o pendrive ou SSD externo e execute:

```text
DoopPortable.exe
```

Não mova apenas o EXE. As pastas `runtime/` e `app/` fazem parte da aplicação.

## Escopo da V1

Incluído:

- operação local e offline do núcleo do Doop;
- PGlite local;
- armazenamento local de assets;
- runtime Bun incluído;
- shell Tauri;
- build portátil em ZIP;
- nenhuma dependência de Docker ou PostgreSQL externo.

Fora do escopo desta versão:

- modelos de IA locais;
- integração com Ollama/llama.cpp;
- atualização automática;
- assinatura Authenticode;
- distribuição macOS/Linux portátil.

## Observações

A máquina Windows ainda precisa do Microsoft Edge WebView2 Runtime, que já vem instalado por padrão em versões atuais do Windows 10 e Windows 11.

Recursos do Doop que dependem de serviços externos continuam indisponíveis quando o computador estiver sem internet. O canvas, banco local e armazenamento local permanecem no dispositivo portátil.
