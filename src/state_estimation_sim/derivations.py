from sympy import symbols, sin, cos, pi, diff, Matrix, pprint, simplify, Symbol

L_x = Symbol("Li.x")
L_y = Symbol("Li.y")
L_theta = Symbol("Li.theta")

Y_d = Symbol("Yi.d")
Y_phi = Symbol("Yi.phi")
Y_alpha = Symbol("Yi.alpha")

X_x = Symbol("X.x")
X_y = Symbol("X.y")
X_theta = Symbol("X.theta")

Var_L_xx = Symbol("Var_Li.xx")
Var_L_xy = Symbol("Var_Li.xy")
Var_L_xt = Symbol("Var_Li.xt")
Var_L_yy = Symbol("Var_Li.yy")
Var_L_yt = Symbol("Var_Li.yt")
Var_L_tt = Symbol("Var_Li.tt")

Var_Y_dd = Symbol("Var_Yi.dd")
Var_Y_pp = Symbol("Var_Yi.pp")
Var_Y_aa = Symbol("Var_Yi.aa")

Var_X_xx = Symbol("Var_X.xx")
Var_X_xy = Symbol("Var_X.xy")
Var_X_xt = Symbol("Var_X.xt")
Var_X_yy = Symbol("Var_X.yy")
Var_X_yt = Symbol("Var_X.yt")
Var_X_tt = Symbol("Var_X.tt")

if False:
    X_x = L_x + Y_d*cos(L_theta + Y_alpha)
    X_y = L_y + Y_d*sin(L_theta + Y_alpha)
    X_theta = L_theta + Y_alpha - Y_phi - pi

    J_X = Matrix((
        ( diff(X_x, L_x),     diff(X_x, L_y),     diff(X_x, L_theta),     diff(X_x, Y_d),     diff(X_x, Y_phi),     diff(X_x, Y_alpha)     ),
        ( diff(X_y, L_x),     diff(X_y, L_y),     diff(X_y, L_theta),     diff(X_y, Y_d),     diff(X_y, Y_phi),     diff(X_y, Y_alpha)     ),
        ( diff(X_theta, L_x), diff(X_theta, L_y), diff(X_theta, L_theta), diff(X_theta, Y_d), diff(X_theta, Y_phi), diff(X_theta, Y_alpha) )
    ))

    Vars = Matrix((
        ( Var_L_xx, Var_L_xy, Var_L_xt, 0,        0,        0        ),
        ( Var_L_xy, Var_L_yy, Var_L_yt, 0,        0,        0        ),
        ( Var_L_xt, Var_L_yt, Var_L_tt, 0,        0,        0        ),
        ( 0,        0,        0,        Var_Y_dd, 0,        0        ),
        ( 0,        0,        0,        0,        Var_Y_pp, 0        ),
        ( 0,        0,        0,        0,        0,        Var_Y_aa )
    ))

    # pprint(J_X, use_unicode=False)
    result = simplify(J_X*Vars*J_X.transpose())
    print("xx: " + str(result[0, 0]) + ",")
    print("xy: " + str(result[0, 1]) + ",")
    print("xt: " + str(result[0, 2]) + ",")
    print("yy: " + str(result[1, 1]) + ",")
    print("yt: " + str(result[1, 2]) + ",")
    print("tt: " + str(result[2, 2]) + ",")

if True:
    L_x = X_x + Y_d*cos(X_theta + Y_phi)
    L_y = X_y + Y_d*sin(X_theta + Y_phi)
    L_theta = X_theta + Y_phi + pi - Y_alpha

    J_X = Matrix((
        ( diff(L_x, Y_d),     diff(L_x, Y_phi),     diff(L_x, Y_alpha),     diff(L_x, X_x),     diff(L_x, X_y),     diff(L_x, X_theta)     ),
        ( diff(L_y, Y_d),     diff(L_y, Y_phi),     diff(L_y, Y_alpha),     diff(L_y, X_x),     diff(L_y, X_y),     diff(L_y, X_theta)     ),
        ( diff(L_theta, Y_d), diff(L_theta, Y_phi), diff(L_theta, Y_alpha), diff(L_theta, X_x), diff(L_theta, X_y), diff(L_theta, X_theta) )
    ))

    Vars = Matrix((
        ( Var_Y_dd, 0,        0,        0,        0,        0        ),
        ( 0,        Var_Y_pp, 0,        0,        0,        0        ),
        ( 0,        0,        Var_Y_aa, 0,        0,        0        ),
        ( 0,        0,        0,        Var_X_xx, Var_X_xy, Var_X_xt ),
        ( 0,        0,        0,        Var_X_xy, Var_X_yy, Var_X_yt ),
        ( 0,        0,        0,        Var_X_xt, Var_X_yt, Var_X_tt ),
    ))

    # pprint(J_X, use_unicode=False)
    result = simplify(J_X*Vars*J_X.transpose())
    print("xx: " + str(result[0, 0]) + ",")
    print("xy: " + str(result[0, 1]) + ",")
    print("xt: " + str(result[0, 2]) + ",")
    print("yy: " + str(result[1, 1]) + ",")
    print("yt: " + str(result[1, 2]) + ",")
    print("tt: " + str(result[2, 2]) + ",")