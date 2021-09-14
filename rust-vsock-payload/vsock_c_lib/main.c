//
// Copyright (c) 2021 Intel Corporation
//
// SPDX-License-Identifier: BSD-2-Clause-Patent
//
// This example does the same work as rust-vsock-payload/src/server.rs
//

#include <stddef.h>

typedef long int ssize_t;

typedef unsigned int socklen_t;

struct sockaddr {
	unsigned short svm_family;
	unsigned short svm_reserved1;
	unsigned int   svm_port;
	unsigned int   svm_cid;
	unsigned char  svm_flags;
	unsigned char  svm_zero[3];
};

extern int     socket (int family, int type, int protocol);
extern int     bind (int sockfd, const struct sockaddr *addr,socklen_t addrlen);
extern int     listen (int sockfd, int backlog);
extern int     accept (int sockfd, struct sockaddr *addr, socklen_t *addrlen);
extern ssize_t recv (int sockfd, void *buf, size_t len, int flags);
extern int     connect (int sockfd, const struct sockaddr *addr,socklen_t addrlen);
extern ssize_t send (int sockfd, void *buf, size_t len, int flags);
extern int     shutdown (int fd, int how);

#define RECV_BUF_LEN 1024

int server_entry () {
  int             sockfd;
  unsigned char   buf[1024];
  struct sockaddr bindAddr;
  socklen_t       bindAddrLen;
  struct sockaddr acceptAddr;
  socklen_t       acceptAddrLen;

  bindAddr.svm_port = 1234;
  bindAddr.svm_cid  = 33;
  bindAddrLen       = 11;

  sockfd = socket (0, 0, 0);

  if (bind (sockfd, &bindAddr, bindAddrLen)) {
    return -1;
  }

  if (listen (sockfd, 1)) {
    return -1;
  }

  if (accept (sockfd, &acceptAddr, &acceptAddrLen)) {
    return -1;
  }

  while (1) {
    if (!recv (sockfd, buf, 1024, 0)) {
      break;
    }
  }

  if (shutdown (sockfd, 0)) {
    return -1;
  }
  return 0;
}



int client_entry () {
  int             sockfd;
  struct sockaddr serverAddr;
  socklen_t       serverAddrLen;

  sockfd = socket (0, 0, 0);

  serverAddr.svm_port = 1234;
  serverAddr.svm_cid  = 2;
  serverAddrLen       = 11;

  if (connect (sockfd, &serverAddr, serverAddrLen)) {
      return -1;
  }

  if (!send (sockfd, "hello", 5, 0)) {
    return -1;
  }

  if (shutdown (sockfd, 0)) {
    return -1;
  }
  return 0;
}
