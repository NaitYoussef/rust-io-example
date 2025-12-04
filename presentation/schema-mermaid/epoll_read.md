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

   Note right of S: fd surveillés : [fd1,fd2,fd3]
   activate S
   C->>S: send("Hello")
   Note over S: epoll_wait() => [fd3]<br/>read(3, "Hello")<br/>write(3, "Hi")
   S->>C: send("Hi")
   Note right of S: fd surveillés : [fd1,fd2,fd3]
   deactivate S

```
