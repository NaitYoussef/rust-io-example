```mermaid
%%{ init: {"theme": "default",
           "themeVariables": { "wrap": "false", 'fontSize': '24px' },
           "flowchart": { "curve": "default",
                          "markdownAutoWrap":"false",
                          "wrappingWidth": "600" }
           }
}%%
sequenceDiagram;

   participant C as Client
   participant S as Server

   S->>S: listen()
   Note over S: listen_fd=3
   activate S
   C->>S: connect() send a request 
   S->>C: accept() -> client_fd=4
   Note over C: socket(fd=3)<br/>connecté à srv:port
   Note over S: socket(fd=3)<br/>client_fd=4 lié au client
   C->>S: send("Hello")
   S->>C: send("Hi")
   S->>C: close()
   Note over S: close(client_fd=4)
   deactivate S

```
```
Client 1                             Serveur
|                                      |
|                                      |
|                             listen() -> listen_fd=3
|                                      |
|                                      |
|                                      |
|                                      |
|-------------- connect() ------------>|
|                                      |
|                             accept() -> client_fd=4
|<------------- accept() ok -----------|
|                                      |
|   socket(fd=3)                       |  listen_fd=3
|   connecté à srv:port                |  client_fd=4 lié au client
|                                      |
|----------- send("Hello") ----------->|
|                                      |
|<---------- send("Hi!") --------------|
|                                      |
|                                      |
|------------ close() ----------------->|
|                                      |
|                          close(client_fd=4)
|                                      |
|                                      |
```

