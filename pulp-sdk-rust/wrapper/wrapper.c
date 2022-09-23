// Wrapper for some function declared as static in header files in the pulp-sdk
#include "stdint.h"
#include <pmsis.h>
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
    pi_cl_dma_cmd(ext, loc,size,dir, cmd);
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

struct pi_cluster_task *pi_cluster_task_wrap(
    struct pi_cluster_task *task, 
    void (*entry)(void*), 
    void *arg) {

  return pi_cluster_task(task, entry, arg);
}

// L1
void *pmsis_l1_malloc_wrap(uint32_t size) {
  return pmsis_l1_malloc(size);
  // return pi_cl_l1_malloc((void *) 0, size);
}

void pmsis_l1_free_wrap(void *chunk, uint32_t size) {
  // return pmsis_l1_malloc_free(chunk);
  return;
}

// L2
void *pmsis_l2_malloc_wrap(uint32_t size) {
  return pmsis_l2_malloc(size);
}

void pmsis_l2_free_wrap(void *chunk, uint32_t size) {
  // return pmsis_l2_malloc_free(chunk);
  return;
}

int pi_core_id_wrap() {
  return pi_core_id();
}

void led_turn_on() {
  // #define LED_PIN 2
  // Open Led
  static pi_device_t led_gpio_dev;  
  // Initialize the LED pin
  pi_gpio_pin_configure(&led_gpio_dev, 2, PI_GPIO_OUTPUT);
  pi_gpio_pin_write(&led_gpio_dev, 2, 1);

}

void led_turn_off() {
  // #define LED_PIN 2
  // Open Led
  static pi_device_t led_gpio_dev;
  // Initialize the LED pin
  pi_gpio_pin_configure(&led_gpio_dev, 2, PI_GPIO_OUTPUT);
  pi_gpio_pin_write(&led_gpio_dev, 2, 0);
}
