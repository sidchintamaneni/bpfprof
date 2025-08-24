LINUX := /linux
SOURCES := $(wildcard *.kern.c)
FILES := $(SOURCES:.c=.o)
USER_SRC := $(wildcard *.user.c)
USER := $(USER_SRC:.c=)
BPF-CLANG := clang
BPF_CLANG_CFLAGS := -target bpf -g -Wall -O2 -c
INCLUDE := -I$(LINUX)/usr/include/ -I$(LINUX)/tools/lib/
USER-CFLAGS := -I$(LINUX)/usr/include -I$(LINUX)/tools/lib/ -L$(LINUX)/tools/lib/bpf/

all: $(FILES) $(USER)
	
$(FILES) : %.o : %.c
	$(BPF-CLANG) $(INCLUDE) $(BPF_CLANG_CFLAGS) -o $@ $<

$(USER) : % : %.c
	clang $(USER-CFLAGS) $< -lbpf -o $@


.PHONY : clean

clean :
	rm $(FILES) $(USER)
