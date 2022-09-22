
#include "pmsis.h"
#include <bsp/bsp.h>


#define HOTTING 1
#define REPEAT  3
#define STACK_SIZE 2048
#define LEN (131072)
#define BUF_LEN (14336*3)


PI_L2 char data[LEN];
PI_L2 char data2[LEN];
PI_L1 char key[32];
PI_L1 char iv[12];
PI_L2 int lennn[1];
PI_L2 struct pi_device ram;
PI_L2 uint32_t ram_ptr;


extern void *cluster_init();

extern void cluster_close(void* wrapper);

extern void encrypt(char *data, size_t len, char *key, char *iv, void* wrapper, pi_device_t* ram, int cipher);

extern void encrypt_serial_orig(char *data, size_t len, char *key, char *iv);

void test(uint8_t* a,  uint8_t* b, uint8_t* c, uint32_t len);

#define INIT_STATS()  

#define ENTER_STATS_LOOP()  \
    unsigned long _cycles = 0; \
    unsigned long _instr = 0; \
    unsigned long _active = 0; \
    unsigned long _ldext = 0; \
    unsigned long _tcdmcont = 0; \
    unsigned long _ld = 0; \
    unsigned long _st = 0; \
    unsigned long _ldstall = 0; \
    unsigned long _imiss = 0; \
    for(int _k=0; _k<HOTTING+REPEAT; _k++) { \
      pi_perf_conf((1<<PI_PERF_CYCLES) | (1<<PI_PERF_INSTR) | (1<<PI_PERF_ACTIVE_CYCLES) | (1<<PI_PERF_LD) | (1<<PI_PERF_ST) | (1<<PI_PERF_JR_STALL) | (1<<PI_PERF_JR_STALL) );


#define START_STATS()  \
    pi_perf_reset(); \
    pi_perf_start();

#define STOP_STATS() \
     pi_perf_stop(); \
     if (_k >= HOTTING) \
      { \
        _cycles   += pi_perf_read (PI_PERF_CYCLES); \
        _instr    += pi_perf_read (PI_PERF_INSTR); \
    	_active   += pi_perf_read (PI_PERF_ACTIVE_CYCLES); \
        _ld    += pi_perf_read (PI_PERF_LD); \
        _st    += pi_perf_read (PI_PERF_ST); \
    	_ldstall  += pi_perf_read (PI_PERF_JR_STALL); \
        _imiss    += pi_perf_read (PI_PERF_JR_STALL); \
      }

#define EXIT_STATS_LOOP()  \
    } \
    printf("[%d] total cycles = %lu\n", 0, _cycles/REPEAT); \
    printf("[%d] instructions = %lu\n", 0, _instr/REPEAT); \
    printf("[%d] active cycles = %lu\n", 0, _active/REPEAT); \
    printf("[%d] loads = %lu\n", 0, _ld/REPEAT); \
    printf("[%d] stores = %lu\n", 0, _st/REPEAT); \
    printf("[%d] LD stalls = %lu\fn", 0, _ldstall/REPEAT); \
    printf("[%d] I$ misses = %lu\n", 0, _imiss/REPEAT);


static void cluster_entry(void *arg)
{
  encrypt(data, lennn[0], key, iv, arg, NULL, 0);  
}


int main()
{
  pi_device_t cluster_dev;
  struct pi_cluster_conf conf;
  struct pi_cluster_task cluster_task = {0};

// open the cluster
  pi_cluster_conf_init(&conf);
  pi_open_from_conf(&cluster_dev, &conf);
  if (pi_cluster_open(&cluster_dev))
  {
    printf("ERROR: Cluster not working\n");
    return -1;
  }
  void* wrapper = cluster_init(&cluster_dev);

  printf("%p\n", wrapper);
  if (wrapper == NULL) {
    exit(2);
  }

  lennn[0] = LEN;

  printf("iteration: %d\n", LEN);
  pi_cluster_task(&cluster_task, cluster_entry, wrapper);
  pi_cluster_send_task_to_cl(&cluster_dev, &cluster_task);
  printf("iteration: %d\n", LEN);
    
  cluster_close(wrapper);
  return 0;
}