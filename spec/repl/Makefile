AGDA=agda
BNFC=bnfc
ALEX=alex
HAPPY=happy
GHC=ghc
GHCFLAGS=-O2

MOUT=MAlonzo
BNFC_STAMP=.bnfc.stamp
AGDA_STAMP=.agda.stamp

.PHONY: all clean
all: repl

# Agda -> Haskell: single stamp file to avoid multi-target issues
$(AGDA_STAMP): ../Language.agda ../Helpers.agda ../spec.agda-lib
	$(AGDA) --ghc --ghc-dont-call-ghc --compile-dir=../_build ../Language.agda
	rm -rf $(MOUT)
	cp -a ../_build/MAlonzo $(MOUT)
	touch $@

MAlonzo/Code/Language.hs: $(AGDA_STAMP)
	@touch $@

# BNFC: one recipe for all generated sources via stamp
$(BNFC_STAMP): Alisp.cf
	$(BNFC) --haskell -d Alisp.cf
	touch $@

Alisp/Abs.hs: $(BNFC_STAMP)
Alisp/Lex.x: $(BNFC_STAMP)
Alisp/Par.y: $(BNFC_STAMP)

Alisp/Lex.hs: Alisp/Lex.x
	$(ALEX) $< -o $@

Alisp/Par.hs: Alisp/Par.y
	$(HAPPY) $< -o $@ --ghc

repl: Main.hs $(AGDA_STAMP) Alisp/Lex.hs Alisp/Par.hs Alisp/Abs.hs Alisp/ErrM.hs
	$(GHC) $(GHCFLAGS) Main.hs Alisp/Abs.hs Alisp/Lex.hs Alisp/Par.hs Alisp/ErrM.hs Alisp/Print.hs -o $@

clean:
	rm -rf repl *.hi *.o Alisp/*.hi Alisp/*.o MAlonzo $(MOUT)/*.hi
	rm -f Alisp/Abs.hs Alisp/Lex.hs Alisp/Par.hs $(BNFC_STAMP) $(AGDA_STAMP)
	rm -rf ../_build/MAlonzo
