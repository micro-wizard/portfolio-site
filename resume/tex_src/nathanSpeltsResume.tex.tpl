% !TEX TS-program = xelatex
% !TEX encoding = UTF-8 Unicode
% -*- coding: UTF-8; -*-
% vim: set fenc=utf-8

%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%
%% CV.tex
%% <https://github.com/zachscrivena/simple-resume-cv>
%% This is free and unencumbered software released into the
%% public domain; see <http://unlicense.org> for details.
%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%

% See "README.md" for instructions on compiling this document.

\documentclass[letterpaper,MMMyyyy,nonstopmode]{simpleresumecv}
% Class options:
% a4paper, letterpaper, nonstopmode, draftmode
% MMMyyyy, ddMMMyyyy, MMMMyyyy, ddMMMMyyyy, yyyyMMdd, yyyyMM, yyyy

%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%
%% PREAMBLE.
%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%

% CV Info (to be customized).
\newcommand{\CVAuthor}{Nathan Spelts}
\newcommand{\CVTitle}{Nathan Spelts' CV}
\newcommand{\CVNote}{}
\newcommand{\CVWebpage}{https://nathanspelts.com/}

% PDF settings and properties.
\hypersetup{
pdftitle={\CVTitle},
pdfauthor={\CVAuthor},
pdfsubject={\CVWebpage},
pdfcreator={XeLaTeX},
pdfproducer={},
pdfkeywords={},
unicode=true,
bookmarks=true,
bookmarksopen=true,
pdfstartview=FitH,
pdfpagelayout=OneColumn,
pdfpagemode=UseOutlines,
hidelinks,
breaklinks}

\pagenumbering{gobble}

% Shorthand.
\newcommand{\Code}[1]{\mbox{\textbf{\#1}}}
\newcommand{\CodeCommand}[1]{\mbox{\textbf{\textbackslash{\#1}}}}

%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%
%% ACTUAL DOCUMENT.
%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%

\begin{document}

%%%%%%%%%%%%%%%
% TITLE BLOCK %
%%%%%%%%%%%%%%%

\Title{\CVAuthor}

(((subtitle)))

\begin{Body}

\Section
{Work Experience}
{Work Experience}
{PDF:Work Experience}

(((work)))

\Section
{Education}
{Education}
{PDF:Education}

(((education)))

\end{Body}
\end{document}
