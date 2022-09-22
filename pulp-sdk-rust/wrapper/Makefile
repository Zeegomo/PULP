io=uart
PMSIS_OS = freertos

# COMPILER FLAGS
APP = wrapper
APP_CFLAGS += -O3 -g
APP_CFLAGS += -DconfigUSE_TIMERS=1 -DINCLUDE_xTimerPendFunctionCall=1
APP_SRCS += main.c wrapper.c 

CONFIG_GAP_LIB_JPEG = 1

RUNNER_CONFIG = $(CURDIR)/config.ini

include $(RULES_DIR)/pmsis_rules.mk
