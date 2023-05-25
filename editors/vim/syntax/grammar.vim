" Vim syntax file
" Language:     Grammar
" Maintainer:   Phoenix
" Last Change:  2021 Aug 3
" Remark:       None

if exists("b:current_syntax")
    finish
endif

syntax case match

syntax match grammarIdentifier /\<\l\w*\>/
syntax match grammarType       /\<\u\+\>/

syntax match grammarOperator /->/
syntax match grammarOperator /|/
syntax match grammarOperator /*/
syntax match grammarOperator /?/


syntax region grammarString start=/"/ end=/"/
syntax region grammarString start=/'/ end=/'/

syntax keyword grammarTodo TODO FIXME XXX NOTE contained
syntax match grammarComment /--.*/ contains=grammarTodo

let b:current_syntax = "grammar"

highlight def link grammarIdentifier Identifier
highlight def link grammarType Type
highlight def link grammarOperator Operator
highlight def link grammarString String
highlight def link grammarTodo Todo
highlight def link grammarComment Comment
