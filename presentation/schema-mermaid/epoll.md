```mermaid
%%{ init: {"theme": "default",
           "themeVariables": { "wrap": "false", 'fontSize': '24px' },
           "flowchart": { "curve": "default",
                          "markdownAutoWrap":"false",
                          "wrappingWidth": "600" }
           }
}%%
sequenceDiagram

   participant C as Client
   participant S as Server

   S->>S: listen()
   Note over S: listen_fd=3<br/>epoll_ctl(ADD,3)
   Note right of S: epoll_wait() ⏳
   C->>S: connect() send a request 
   activate S
   Note over S: epoll_wait() => [3]<br/>accept() -> client_fd=4
   S->>C: accept ok
   Note over C: socket(fd=3)<br/>connecté à srv:port
   Note over S: listen(fd=3)<br/>client_fd=4 lié au client<br/>epoll_ctl(ADD, 4)
   Note right of S: epoll_wait() ⏳
   C->>S: send("Hello")
   Note over S: epoll_wait() => [4]<br/>read(4, "Hello")<br/>write(4, "Hi")
   S->>C: send("Hi")
   Note right of S: epoll_wait() ⏳
   C->>S: close()
   Note over S: epoll_wait() => 4<br/>close(client_fd=4)<br/>epoll_ctl(DEL, 4)
   Note right of S: epoll_wait() ⏳
   deactivate S

```
```
Client 1                              Serveur
|                                      |
|                                      |
|                             listen() -> listen_fd=3
|                                      |
|                             epoll_ctl(ADD, 3)         
|                             epoll_wait() ⏳        
|                                      |
|                                      |
|-------------- connect() ------------>|
|                             epoll_wait() ====> [3]
|                             accept() -> client_fd=4
|<------------- accept() ok -----------|
|                                      |
|   socket(fd=3)                       |  listen_fd=3
|   connecté à srv:port                |  client_fd=4 lié au client
|                              epoll_ctl(ADD, 4)
|                              epoll_wait() ⏳  
|                                      |
|----------- send("Hello") ----------->|
|                              epoll_wait() ====> [4]        
|<---------- send("Hi!") --------------|
|                              epoll_wait() ⏳         
|                                      |
|------------ close() ---------------->|
|                              epoll_wait() ====> [4]        
|                              close(client_fd=4)
|                              epoll_ctl(DEL, 4)        
|                              epoll_wait() ⏳
|                                      |
```

