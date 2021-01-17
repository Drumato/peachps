#include <arpa/inet.h>
#include <errno.h>
#include <linux/if.h>
#include <linux/if_ether.h>
#include <linux/if_packet.h>
#include <stdio.h>
#include <string.h>
#include <sys/ioctl.h>
#include <sys/socket.h>
#include <unistd.h>

typedef struct RawSocket
{
    int32_t fd;
    uint8_t mac_addr[6];
} RawSocket;

int try_open_raw_socket()
{
    int fd = socket(PF_PACKET, SOCK_RAW, htons(ETH_P_ALL));
    return fd;
}
int find_dev_interface_index(struct ifreq *ifr, int dev_fd)
{
    if (ioctl(dev_fd, SIOCGIFINDEX, ifr) == -1)
    {
        return -1;
    }
    return 0;
}

// ソケットにアドレスを割り当てる
int bind_address_to_socket(int dev_fd, struct sockaddr_ll *sock_addr, struct ifreq *ifr)
{
    sock_addr->sll_family = AF_PACKET;
    sock_addr->sll_protocol = htons(ETH_P_ALL);
    sock_addr->sll_ifindex = ifr->ifr_ifindex;

    if (bind(dev_fd, (struct sockaddr *)sock_addr, sizeof(*sock_addr)) == -1)
    {
        return -1;
    }

    return 0;
}

int set_promiscuous_mode(int fd, struct ifreq *ifr)
{
    // set IFF_PROMISC flag
    if (ioctl(fd, SIOCGIFFLAGS, ifr) == -1)
    {
        return -1;
    }
    ifr->ifr_flags = ifr->ifr_flags | IFF_PROMISC;

    if (ioctl(fd, SIOCSIFFLAGS, ifr) == -1)
    {
        return -1;
    }
    return 0;
}

int set_mac_address(char *interface_name, uint8_t *mac_addr, struct ifreq *ifr)
{
    int fd = socket(AF_INET, SOCK_DGRAM, 0);
    if (fd == -1)
    {

        return -1;
    }

    ifr->ifr_addr.sa_family = AF_INET;
    strncpy(ifr->ifr_name, interface_name, sizeof(ifr->ifr_name) - 1);
    if (ioctl(fd, SIOCGIFHWADDR, ifr) == -1)
    {
        close(fd);
        return -1;
    }

    memcpy(mac_addr, ifr->ifr_hwaddr.sa_data, 6);
    close(fd);

    return 0;
}

int _setup_raw_sock(char *interface_name, RawSocket *raw_sock)
{
    struct ifreq ifr;
    struct sockaddr_ll sock_addr;
    memset(&ifr, 0, sizeof(ifr));
    memset(&sock_addr, 0, sizeof(sock_addr));

    if ((raw_sock->fd = try_open_raw_socket()) == -1)
    {
        perror("failed to open raw socket");
        return -1;
    }

    strncpy(ifr.ifr_name, interface_name, sizeof(ifr.ifr_name) - 1);
    if (find_dev_interface_index(&ifr, raw_sock->fd) == -1)
    {
        perror("failed to find network device interface index");
        return -1;
    }

    if (bind_address_to_socket(raw_sock->fd, &sock_addr, &ifr) == -1)
    {
        perror("failed to bind address to raw socket");
        return -1;
    }

    if (set_promiscuous_mode(raw_sock->fd, &ifr) == -1)
    {
        perror("failed to set promiscuous mode");
        return -1;
    }

    // get physical device's mac address
    memset(&ifr, 0, sizeof(ifr));
    if (set_mac_address(interface_name, raw_sock->mac_addr, &ifr) == -1)
    {
        return -1;
    }

    return 0;
}