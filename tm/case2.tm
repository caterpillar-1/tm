#N = 5
#Q = {init,cls,mh,mul,copy1,shiftb,shiftb1,cmp,true,true1,true2,true3,false,false1,false2,false3,false4,halt_accept,halt_reject}
#S = {1}
#G = {_,1,x,X,c,C,t,T,a,A,b,r,u,e,f,l,s}
#q0 = init
#B = _
#F = {halt_accept}

; Tapes:
;   0: t: I/O
;   1: a
;   2: b
;   3: c = a*b
;   4: arg

init    _____   _____   *****   halt_reject
init    1____   111_x   *****   mul

; cls
cls     ****c   ***_c   ***r*   cls
cls     ***_c   ***_c   ***l*   mul
cls     ****t   _***t   r****   cls
cls     _***t   _***t   *****   false
cls     ****T   _***T   r****   cls
cls     _***T   _***T   *****   true

; mh
mh      ***_a   ***_a   *l***   mh
mh      *_*_a   *_*_a   *r***   copy1
mh      ***_A   ***_A   *l***   mh
mh      *_*_A   *_*_b   *r***   mh
mh      ***_b   ***_b   **l**   mh
mh      **__b   **__c   **rl*   mh
mh      ****c   ****c   ***l*   mh
mh      ***_c   ***_c   ***r*   cmp
mh      ****C   ****C   ***l*   mh
mh      ***_C   ***_t   ***r*   mh
mh      ****t   ****t   l****   mh
mh      _***t   _***c   r****   cls
mh      ****x   ****x   l****   mh
mh      _***x   _***t   r****   cls
mh      ****X   ****X   l****   mh
mh      _***X   _***T   r****   cls

; mul
mul     ***_*   ***_*   *****   copy1

; copy1
copy1   *1*_*   *1*1*   *r*r*   copy1
copy1   *_*_*   *_*_*   *****   shiftb

; shiftb
shiftb  *_*_*   *_*_*   **r**   shiftb1
shiftb1 *_1_*   *_1_a   *l***   mh
shiftb1 *___*   *11_A   *ll**   mh

; cmp
cmp     *****   *****   r**r*   cmp
cmp     ***_*   ***_C   ***l*   mh
cmp     _****   _***x   l****   mh
cmp     _**_*   _**_X   l**l*   mh

; true and false
true    _***T   t***T   r****   true1
true1   _***T   r***T   r****   true2
true2   _***T   u***T   r****   true3
true3   _***T   e***T   r****   halt_accept

false   _***t   f***t   r****   false1
false1  _***t   a***t   r****   false2
false2  _***t   l***t   r****   false3
false3  _***t   s***t   r****   false4
false4  _***t   e***t   r****   halt_reject
