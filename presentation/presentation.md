---
title: Voyage au bout des APIs IO de Linux
authors:
 - Jean-Eudes Couignoux
 - Youssef Nait Belkacem
---


Speakers
---

<!-- column_layout: [1, 1] -->

<!-- column: 0 -->

![](images/jean-eudes.jpeg)
_Jean-Eudes Couignoux (capco)_

<!-- column: 1 -->
![](images/youssef.png)
_Youssef Nait Belkacem (freelance)_


<!-- end_slide -->

Sommaire
---

  * Quelques concepts linuxiens
  * Présentation d'api linux
    * select
    * poll
    * epoll / kqueue
    * io uring
  * io uring et le monde Java
  * Conclusion

<!-- end_slide -->

<!-- jump_to_middle -->

Everything is a file
===

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
```
+----------------+         +----------------+         +---------------------------+
|  Appels        |         |      VFS       |         |  +--------------------+   |
|  systèmes      |         | (Virtual File  |         |  | Fichier sur disque |   |
|                |         |   System)      |         |  +--------------------+   |
|  open          | ----->  |                | ----->  |  +--------------------+   |
|  read/write    |         |                | ----->  |  | Socket réseau      |   |
|  fchmod        |         |                | ----->  |  +--------------------+   |
|  ...           |         |                | ----->  |  +--------------------+   |
+----------------+         +----------------+         |  | Pipe / FIFO        |   |
                                                      |  +--------------------+   |
                                                      |  +--------------------+   |
                                                      |  | Fichier virtuel    |   |
                                                      |  | (/proc, /dev)      |   |
                                                      |  +--------------------+   |
                                                      +---------------------------+
```

<!-- end_slide -->

<!-- jump_to_middle -->

Everything is a filedescriptor
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

```
Avant ajout :
+-----------------------------+
|         poll fds[]          |
|-----------------------------|
|  [ fd1 ]  [ fd2 ]  [ fd3 ]  |  <-- tableau de pollfd surveillés
+-----------------------------+

Ajout d'une nouvelle socket (fd4) :
        |
        v
Ajout de fd4 dans le tableau fds[]
        |
        v
poll([fd1, fd2, fd3, fd4])
        |
        v
Après ajout :
+--------------------------------------+
|         poll fds[]                   |
|--------------------------------------|
|  [ fd1 ]  [ fd2 ]  [ fd3 ]  [ fd4 ]  |  <-- tableau de pollfd surveillés
+--------------------------------------+
```

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

```
Avant ajout :
+-----------------------------+
|         epoll instance      |
|-----------------------------|
|  [ fd1 ]  [ fd2 ]  [ fd3 ]  |  <-- file descriptors surveillés
+-----------------------------+

Ajout d'une nouvelle socket (fd4) :
        |
        v
epoll_ctl(ADD, fd4)
        |
        v

Après ajout :
+--------------------------------------+
|         epoll instance               |
|--------------------------------------|
|  [ fd1 ]  [ fd2 ]  [ fd3 ]  [ fd4 ]  |  <-- file descriptors surveillés
+--------------------------------------+
```

<!-- end_slide -->
API epoll - Fin d'une requête
---

```
Avant suppression :
+--------------------------------------+
|         epoll instance               |
|--------------------------------------|
|  [ fd1 ]  [ fd2 ]  [ fd3 ]  [ fd4 ]  |  <-- file descriptors surveillés
+--------------------------------------+

Suppression d'une socket (fd3) :
        |
        v
epoll_ctl(DEL, fd3)
        |
        v

Après suppression :
+-----------------------------+
|         epoll instance      |
|-----------------------------|
|  [ fd1 ]  [ fd2 ]  [ fd4 ]  |  <-- file descriptors surveillés
+-----------------------------+
```

<!-- end_slide -->
API io uring, caractéristiques
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
+---------------------------------------------------------------------+
|                              user space                             |
|                                                                     |
|                   +-------------------------------+                 |
|                   |     read / write, liburing    |                 |
|                   +-------------------------------+                 |
|                        |                    ^                       |
|                        v                    |                       |
|        +-------------------+    +-------------------+               |
|        | Submission Queue  |    | Completion Queue  |               |
|        |   [SQE 1]         |    |   [CQE 1]         |               |
|--------|   [SQE 2]         |----|   [CQE 2]         |---------------|
|        |   [SQE 3]         |    |   [CQE 3]         |               |
|        +-------------------+    +-------------------+               |
|                |                        ^                           |
|                |                        |                           |
|                +----------+  +----------+                           |
|                           |  |                                      |
|                         +--------+                                  |
|                         | readv  |                                  |
|                         +--------+                                  |
|                                                                     |
|                        Kernel space                                 |
+---------------------------------------------------------------------+
```
<!-- end_slide -->

Comparatif des performances
---

<!-- end_slide -->

IO uring et l'écosytème java
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
