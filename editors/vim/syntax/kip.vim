" Vim syntax file
" Language:     Kip
" Maintainer:   Phoenix
" Last Change:  2021 Jun 28
" Remark: None
"
if exists("b:current_syntax")
    finish
endif

syntax case match
syntax keyword kipKeyword func var ret
syntax keyword kipType s32 s64 u32 u64
syntax keyword kipType str
syntax match kipNumber /\<\d\+\>/
syntax match kipIdentifier /\<\h\w*\>/
syntax match kipPreProc /@\<\h\w*\>/
" Section: Operators --- {{{
syntax match kipOperator /=/
syntax match kipOperator /==/
syntax match kipOperator /\*/
syntax match kipOperator /\//
syntax match kipOperator /+/
syntax match kipOperator /-/
syntax match kipOperator /%/
syntax match kipOperator />/
syntax match kipOperator /</
" }}}

syntax match kipFunction /\<\h\w*\>(/he=e-1,me=e-1
" type annotations
syntax region kipType start=/:\s*/ end=/\<\h\w*\>/
syntax region kipString start=/"/ end=/"/

syntax keyword kipTodo TODO FIXME XXX contained
syntax match kipLineComment /\/\/.*/ contains=kipTodo
syntax region kipBlockComment start="/\*" end="\*/" contains=kipTodo

let b:current_syntax = "kip"

highlight def link kipNumber Number
highlight def link kipTodo Todo
highlight def link kipString String
highlight def link kipKeyword Keyword
highlight def link kipType Type
highlight def link kipPreProc PreProc
highlight def link kipIdentifier Identifier
highlight def link kipFunction Function
highlight def link kipOperator Operator
highlight def link kipLineComment Comment
highlight def link kipBlockComment Comment
