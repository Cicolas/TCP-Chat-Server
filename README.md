# Chat Server
---
Servidor de Chat síncrono implementado com tokio em Rust através de conexões TCP

### Como Usar
---
Basta rodar:

`> cargo run`

Após isso seu servidor estará rodando na rota

`127.0.0.1:7777`

Qualquer conexão TCP que for estabelessida já estará pronta para receber e mandar mensagens.

há duas "mensagens" pricipais que podem serem enviadas ao servidor:

```json
// todo cliente deve se registrar para que seja possivel indentificar na hora de enviar mnsagens
{
    event: "register",
    content: {
        name: name,
    }
}

// toda mensagem pode ser enviada da seguinte maneira: (O usuário fica restrito a qual IP acessou e registrou anteriormente)
{
    event: "message",
    content: {
        message: "bla bla bla",
    }
} 
```