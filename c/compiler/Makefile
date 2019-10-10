CFLAGS = -O2

base_cflags := -std=c11 -Wall -Wextra -Wpointer-arith -Wwrite-strings -Wmissing-prototypes -Wdouble-promotion -Werror=implicit-function-declaration -Winit-self -Wstrict-prototypes

override CFLAGS := $(base_cflags) $(CFLAGS)

ifdef ASAN
override CFLAGS += '-fsanitize=address'
override LIBS += '-fsanitize=address'
endif

objs := sexpr_token.o sexpr_token_test.o testutil.o

all: sexpr_token_test
clean:
	rm -f $(objs) $(patsubst %.o,%.d,$(objs))
.PHONY: all clean

-include: $(patsubst %.o,%.d,$(objs))

%.o: %.c
	$(CC) -MF $(patsubst %.o,%.d,$@) -MMD -MP $(CFLAGS) -c -o $@ $<

sexpr_token_test: sexpr_token.o sexpr_token_test.o testutil.o
	$(CC) $(LDFLAGS) -o $@ $^ $(LIBS)
