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
   Note over S: listen_fd=3<br/>ring_submissing(Accept:3, UD:A)
   Note right of S: submit_and_wait() ⏳
   C->>S: connect() send a request 
   activate S
   Note over S: submit_and_wait() ====> [(UD:A, 4)]
   S->>C: accept ok
   Note over C: socket(fd=3)<br/>connecté à srv:port
   Note over S: listener_fd=3 lié au listener<br/>client_fd=4 lié au client<br/>ring_submissing(Accept:3, UD:A)<br/>ring_submission(READ:4, UD:B)
   Note right of S: ring_submit_and_wait() ⏳
   C->>S: send("Hello")
   Note over S: submit_and_wait() ====> [(UD:B, 5)] le 5 correspond à la taille à lire du buffer<br/>ring_submissing(Accept:3, UD:A)<br/>ring_submissing(Write:4, UD:C)
   Note right of S: ring_submit_and_wait() ⏳
   S->>C: send("Hi")
   Note over S: submit_and_wait() ====> [(UD:C)] événement de retour de notre écrite<br/>ring_submission(READ:4, UD:B)
   Note right of S: ring_submit_and_wait() ⏳
   C->>S: close()
   Note over S: submit_and_wait() ====> [UD:B, 0]<br/>close(client_fd=4)
   Note right of S:submit_and_wait() ⏳
   deactivate S

```
```
Client 1                              Serveur
|                                      |
|                                      |
|                             listen() -> listen_fd=3
|                                      |
|                             ring_submissing(Accept:3, UD:A)         
|                             submit_and_wait() ⏳        
|                                      |
|                                      |
|-------------- connect() ------------>|
|                             submit_and_wait() ====> [(UD:A, 4)]
|<------------- accept() ok -----------|
|                                      |
|   socket(fd=3)                       |  listen_fd=3
|   connecté à srv:port                |  client_fd=4 lié au client                           
|                              ring_submissing(Accept:3, UD:A)
|                              ring_submission(READ:4, UD:B)
|                              ring_submit_and_wait() ⏳
|                                      |
|----------- send("Hello") ----------->|
|                              submit_and_wait() ====> [(UD:B, 5)] le 5 correspond à la taille à lire du buffer      
|                              ring_submissing(Accept:3, UD:A)
|                              ring_submissing(Write:4, UD:C)
|                                      |
|                              ring_submit_and_wait() ⏳
|<---------- send("Hi!") --------------|
|                              submit_and_wait() ====> [(UD:C)] événement de retour de notre écrite
|                              ring_submission(READ:4, UD:B)     
|                              submit_and_wait() ⏳
|                                      |
|------------ close() ---------------->|
|                              submit_and_wait() ====> [UD:B, 0]       
|                              close(client_fd=4)
|                              submit_and_wait() ⏳
```