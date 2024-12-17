#N = 4
#Q = {init,copya,copyb,mha,copyc,shiftb0,shiftb1,cls0,cls1,halt_accept,halt_reject,illegal0,illegal1,illegal2,illegal3,illegal4,illegal5,illegal6,illegal7,illegal8,illegal9,illegal10,illegal11,illegal12}
#S = {a,b}
#G = {a,b,c,x,_,i,r,l,e,g,a,n,p,u,t}
#q0 = init
#B = _
#F = {halt_accept}

; Spec:
;   1. init program inits all tapes
;   2. all subprogram should call other subprograms with all heads not in blank
;       (except cls)
;   3. tapes
;       0: I/O
;       1: marka
;       2: markb
;       3: arg

; init
init    a___    a_xx    ****    copya
init    ____    _xxx    ****    illegal0
init    b___    bxxi    ****    cls0

; copya
copya   a_**    aa**    rr**    copya
copya   __**    __xi    ll**    cls0
copya   b_**    b__x    *l**    copyb

; copyb
copyb   b*_*    b*b*    r*r*    copyb
copyb   _*_*    _*_r    l*l*    cls0
copyb   a*_*    axli    ****    cls0

; mha
mha     _a**    _a**    *l**    mha
mha     __**    __**    *r**    copyc

; copyc
copyc   _a**    ca**    rr**    copyc
copyc   __**    __**    *l**    shiftb0

; shiftb
shiftb0 _***    _***    **l*    shiftb1
shiftb1 _*_*    _*_*    ****    halt_accept
shiftb1 _*b*    _*b*    ****    mha

; cls
;   move head 0 left and clear tape 0
;   jump to illegal (i) or mha (r)
cls0    ****    ****    l***    cls0
cls0    _***    _***    r***    cls1
cls1    ****    _***    r***    cls1
cls1    _**i    _**i    ****    illegal0
cls1    _**r    _**r    ****    mha

; func: illegal
illegal0    _***    i***    r***    illegal1
illegal1    _***    l***    r***    illegal2
illegal2    _***    l***    r***    illegal3
illegal3    _***    e***    r***    illegal4
illegal4    _***    g***    r***    illegal5
illegal5    _***    a***    r***    illegal6
illegal6    _***    l***    r***    illegal7
illegal7    _***    _***    r***    illegal8
illegal8    _***    i***    r***    illegal9
illegal9    _***    n***    r***    illegal10
illegal10   _***    p***    r***    illegal11
illegal11   _***    u***    r***    illegal12
illegal12   _***    t***    ****    halt_reject
