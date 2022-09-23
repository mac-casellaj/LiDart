# Makefile
# From https://danielkaes.wordpress.com/2009/03/14/compiling-latex-documents-using-makefiles/

TEX=pdflatex -quiet
BIBTEX=bibtex

all:
	$(TEX) SRS.tex
	$(BIBTEX) SRS
	$(TEX) SRS.tex
	$(TEX) SRS.tex

	$(TEX) CA.tex
	$(BIBTEX) CA
	$(TEX) CA.tex
	$(TEX) CA.tex

clean-all:
	rm -f *.dvi *.log *.bak *.aux *.bbl *.blg *.idx *.ps *.eps *.pdf *.toc *.out *~

clean:
	rm -f *.log *.bak *.aux *.bbl *.blg *.idx *.toc *.out *.synctex.gz *~