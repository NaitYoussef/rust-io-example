---
title: Voyage au bout des APIs IO de Linux
authors:
  - Jean-Eudes Couignoux
  - Youssef Nait Belkacem
---


Speakers
---

<!-- column_layout: [1, 1] -->

<!-- alignment: center -->

<!-- column: 0 -->

![](images/jean-eudes.jpeg)
_Jean-Eudes Couignoux (capco)_

<!-- column: 1 -->
![](images/youssef.jpeg)
_Youssef Nait Belkacem (freelance)_


<!-- end_slide -->

Sommaire
---
* Introduction
* Quelques concepts linuxiens
* Présentation d'API linux
    * select
    * poll
    * epoll / kqueue
    * IO uring
* IO uring et le monde Java
* Conclusion

<!-- end_slide -->
Introduction
---
Le WEB aujourd'hui
----
<!-- column_layout: [1, 1] -->
<!-- column: 0 -->
![img.png](images/social-networks.png)

<!-- column: 1 -->
![img.png](images/trading.png)

<!-- end_slide -->
Introduction
---
Le WEB aujourd'hui
----
<!-- column_layout: [1, 1] -->
<!-- column: 0 -->
![img.png](schema/websocket-1.png)
<!-- column: 1 -->

![img.png](schema/sse-1.png)
<!-- end_slide -->
Introduction
---
<!-- jump_to_middle -->

Everything is a file
===

<!-- end_slide -->

Api FileInputStream en java
===

<!-- jump_to_middle -->

![img.png](images/inputstream.png)

<!-- end_slide -->

Exemple simple
---

``` bash
cat hello.txt
```

``` bash 
...
openat(AT_FDCWD, "presentation/demo.txt", O_RDONLY) = 3
...
read(3, "Bonjour Paris JUG", 262144)    = 17
write(1, "Bonjour Paris JUG", 17Bonjour Paris JUG)       = 17
read(3, "", 262144)                     = 0
...
close(3)                                = 0
close(1)                                = 0
close(2)                                = 0
```

<!-- end_slide -->

Un exemple de Filedescriptor
---

```
pos:    256
flags:  02000002
mnt_id: 18
ino:    1091
```


<!-- end_slide -->
Objets linux représentés par un file descriptor
---

* fichier
* fichier virtuel (/proc, ...)
* socket
* terminal (0, 1 et 2)
* timers
* signaux
* process
* pipe nommé
* ...

<!-- end_slide -->
Linux Virtual File System
---

![img.png](schema/vfs-1.png)

<!-- end_slide -->

<!-- jump_to_middle -->

Everything is a filedescriptor
===

<!-- end_slide -->
Exercice
---

![img.png](schema/exercice-1.png)
<!-- end_slide -->
Exercice
---
<!-- column_layout: [1, 1] -->
<!-- column: 0 -->

![img.png](schema/exercice-multi-client-1.png)
<!-- column: 1 -->
![img.png](schema/exemple-standart-1.png)
<!-- end_slide -->
<!-- jump_to_middle -->
Demo
===
<!-- end_slide -->
API poll, caractéristiques
---

* Api posix (1)
* intégré au kernel dans la version 2.1.23 (1997)
* permet de surveiller plusieurs file descriptor
* quelques problèmes de scalabilité

<!-- end_slide -->
API poll, schéma
---

![img.png](schema/poll_add-1.png)

<!-- end_slide -->
API poll, schéma
---
<!-- column_layout: [1, 1] -->

<!-- column: 0 -->
![img.png](schema/exercice-multi-client-1.png)
<!-- column: 1 -->
![img.png](schema/poll-1.png)

<!-- end_slide -->
<!-- jump_to_middle -->
Demo
===
<!-- end_slide -->
Trade-offs
---

OK

* Traitement des sockets client que lorsqu'on reçoit des événements (accept, read)

KO

* Compléxité en O(n) parcourt intégrale de toute la liste des fds
* Copie Userspace <-> Kernelspace à chaque appel à poll
* Enrigistrement des fds côté kernel à chaque appel poll

<!-- end_slide -->
API epoll, caractéristiques
---

* Api non posix (uniquement sous linux)
* intégré au kernel dans la version 2.5.44 (2002)
* permet de surveiller plusieurs file descriptor
* utilise une queue intégré au noyau pour gérer les évênements

<!-- end_slide -->
API epoll - Réception d'une nouvelle requête
---

![img.png](schema/epoll_add-1.png)

<!-- end_slide -->
API epoll - Fin d'une requête
---

![img.png](schema/epoll_del-1.png)
<!-- end_slide -->
API epoll - Attendre un évènement sur un file descriptor
---

![img.png](schema/epoll_read-1.png)
<!-- end_slide -->
API epoll
---
<!-- column_layout: [1, 1] -->

<!-- column: 0 -->
![img.png](schema/exercice-multi-client-1.png)
<!-- column: 1 -->
![img.png](schema/epoll-1.png)
<!-- end_slide -->
<!-- jump_to_middle -->
Demo
===
<!-- end_slide -->
Trade-offs
---

OK

* Compléxité en O(1) intérvetion uniquement sur les fds concernés
* Enrigistrement des fds à surveiller une seule fois

KO

* Les appels à read et write provequent des copies Userspace <-> Kernelspace
<!--- end_slide -->
API poll et epoll en java
---

Existe depuis java 1.6 avec la classe ```SelectorProvider```.

L'implementation se choisit avec ```java.nio.channels.spi.SelectorProvider```
<!-- column_layout: [1, 1] -->

<!-- column: 0 -->

``` java
Selector s = Selector.open();
ServerSocketChannel s = ServerSocketChannel.open();
s.configureBlocking(false);
s.bind(new InetSocketAddress("0.0.0.0", 8000));
s.register(s, SelectionKey.OP_ACCEPT);
while (true) {
  selector.select();
  Iterator<SelectionKey> keys = s.selectedKeys().iterator();
  while (keys.hasNext()) {
    SelectionKey key = keys.next();
    keys.remove();
```

<!-- column: 1 -->

``` java
(key.isAcceptable()) {
 .... 
else if (key.isReadable()) {
SocketChannel c = (SocketChannel) key.channel();
ByteBuffer buffer = ByteBuffer.allocateDirect(512);
int bytesRead = c.read(buffer);
if (bytesRead == -1) {
  c.close();
} else {
 buffer.flip();
String message = new String(buffer.array(), 0, buffer.limit());
c.write(ByteBuffer.wrap((message + "\n").getBytes()));
}
```

<!-- reset_layout -->


<!-- end_slide -->

API IO uring, caractéristiques
---

* Api non posix (uniquement sous linux)
* intégré au kernel dans la version 5.1 (2019)
* interface asynchrone permettant de manipuler les IO
* zero copy
* limite le nombre d'appel kernel

<!-- end_slide -->
API IO uring (schéma)
---
```
|                          user space                   |
|               +-------------------------------+       |
|               |     read / write, liburing    |       |
|               +-------------------------------+       |
|                    |                    ^             |
|                    v                    |             |
|    +-------------------+    +-------------------+     |
|    | Submission Queue  |    | Completion Queue  |     |
|    |   [SQE 1]         |    |   [CQE 1]         |     |
|----|   [SQE 2]         |----|   [CQE 2]         |-----|
|    +-------------------+    +-------------------+     |
|            |                        ^                 |
|            +----------+  +----------+                 |
|                       v  |                            |
|                     +--------+                        |
|                     | readv  |                        |
|                     +--------+                        |
|                    Kernel space                       |
```
<!-- end_slide -->
API IO uring (schéma)
---
![img.png](schema/io-uring_add-1.png)
<!-- end_slide -->
API IO uring
---
<!-- column_layout: [1, 1] -->

<!-- column: 0 -->

![img.png](schema/exercice-multi-client-1.png)
<!-- column: 1 -->

![img.png](schema/iouring-1.png)
<!-- end_slide -->
<!-- jump_to_middle -->
Demo
===
<!-- end_slide -->
Comparatif des performances (select - poll - epoll - kqueue)
---

![](images/epoll_vs_poll.png)
https://monkey.org/~provos/libevent/libevent-benchmark2.jpg

<!-- end_slide -->
Trade-offs
---

OK

* Pas de copie Kernelspace <-> Userspace

KO

* Un peu plus compliqué à mettre en place côté code
<!-- end_slide -->
Comparatif des performances (iouring)
---

![](images/postgrebench.png)
https://www.postgresql.org/message-id/uvrtrknj4kdytuboidbhwclo4gxhswwcpgadptsjvjqcluzmah%40brqs62irg4dt
<!-- end_slide -->
IO uring et l'écosystème java
---

* intégré via le projet panama
* disponible avec netty (4.2)
    * vertx
    * quarkus (experimental)
    * micronaut

<!-- end_slide -->

Conclusion
---

<!-- end_slide -->


Merci
===

<!-- column_layout: [1, 1] -->

<!-- alignment: center -->

<!-- column: 0 -->

![](images/devfest_dijon.png)
FeedBack
<!-- column: 1 -->
![](images/lien_presentation.png)
Lien vers la présentation
