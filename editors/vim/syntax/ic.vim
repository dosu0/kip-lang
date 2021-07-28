" Vim syntax file
" Language:     Ic
" Maintainer:   Phoenix
" Last Change:  2021 Jul 27
" Remark:       None

if exists("b:current_syntax")
    finish
endif

syntax case match
syntax keyword icStatement goto call arg ret
syntax keyword icConditional ifnz ifz
syntax match icNumber /\<\d\+\>/
syntax match icIdentifier /\<\h\w*\>/
syntax match icStatement /@\<\h\w*\>/
syntax match icLabel /\<\h\w*\>:/
" Section: Operators --- {{{
syntax match icOperator /:=/
syntax match icOperator /==/
syntax match icOperator /\*/
syntax match icOperator /\//
syntax match icOperator /+/
syntax match icOperator /-/
syntax match icOperator /%/
syntax match icOperator />/
syntax match icOperator />=/
syntax match icOperator /<=/
" }}}

syntax region icString start=/'/ end=/'/

syntax keyword icTodo TODO FIXME XXX contained
syntax match icComment /;.*/ contains=icTodo

let b:current_syntax = "ic"

highlight def link icNumber Number
highlight def link icLabel Label
highlight def link icString String
highlight def link icTodo Todo
highlight def link icKeyword Keyword
highlight def link icStatement Statement
highlight def link icConditional Conditional
highlight def link icIdentifier Identifier
highlight def link icOperator Operator
highlight def link icComment Comment
