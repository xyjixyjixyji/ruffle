AS_FILES := $(wildcard *.as)
SWF_FILES := $(AS_FILES:.as=.swf)

all: $(SWF_FILES)

%.swf: %.as
	mxmlc -o $@ -debug $<

clean:
	rm -f *.swf