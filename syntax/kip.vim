if exists("b:current_syntax")
    finish
endif

syntax keyword kipKeyword func var
syntax match kipNumber "[0-9]\+"
syntax match kipOperator "="
syntax match kipOperator "\*"
syntax match kipOperator "/"
syntax match kipOperator "+"
syntax match kipOperator "-"
syntax match kipFunction "\w\(\w\)*("he=e-1,me=e-1
syntax region kipLineComment start="//" end="$"
syntax region kipBlockComment start="/\*" end="\*/"

let b:current_syntax = "kip"

highlight link kipKeyword Keyword
highlight link kipNumber Number
highlight link kipFunction Function
highlight link kipOperator Operator
highlight link kipLineComment Comment
highlight link kipBlockComment Comment
