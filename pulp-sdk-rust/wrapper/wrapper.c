// Wrapper for some function declared as static in header files in the pulp-sdk
#include "stdint.h"
#include "pmsis.h"
#include <bsp/bsp.h>


void pi_cl_team_fork_wrap(int nb_cores, void (*entry)(void *), void *arg)
{
    pi_cl_team_fork(nb_cores, entry, arg);
}

void pi_cl_team_barrier_wrap()
{
    pi_cl_team_barrier();
}

void pi_cl_dma_cmd_wrap(uint32_t ext, uint32_t loc, uint32_t size, pi_cl_dma_dir_e dir, pi_cl_dma_cmd_t *cmd) {
    pi_cl_dma_cmd(ext, loc, size, dir, cmd);
}

void abort_all(){
  exit(1);
}

void pi_cl_dma_wait_wrap(void* copy){
  pi_cl_dma_wait(copy);
}

void pi_cl_ram_read_wait_wrap(pi_cl_ram_req_t* req) {
  pi_cl_ram_read_wait(req);
}

void pi_cl_ram_write_wait_wrap(pi_cl_ram_req_t* req) {
  pi_cl_ram_write_wait(req);
}

void pi_cl_ram_read_wrap( 	struct pi_device *  	device,
		uint32_t  	pi_ram_addr,
		void *  	addr,
		uint32_t  	size,
		pi_cl_ram_req_t *  	req 
	){
  pi_cl_ram_read(device, pi_ram_addr, addr, size, req);
}

void pi_cl_ram_write_wrap( 	struct pi_device *  	device,
		uint32_t  	pi_ram_addr,
		void *  	addr,
		uint32_t  	size,
		pi_cl_ram_req_t *  	req 
	){
  pi_cl_ram_write(device, pi_ram_addr, addr, size, req);
}


int pi_core_id_wrap() {
  return pi_core_id();
}

struct pi_cluster_task *pi_cluster_task_wrap(struct pi_cluster_task *task, void (*entry)(void*), void *arg) {
  return pi_cluster_task(task, entry, arg);
}