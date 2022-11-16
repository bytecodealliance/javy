var Rr = Object.defineProperty, Vr = Object.defineProperties;
var $r = Object.getOwnPropertyDescriptors;
var An = Object.getOwnPropertySymbols;
var qr = Object.prototype.hasOwnProperty, zr = Object.prototype.propertyIsEnumerable;
var Mn = (n, r, t) => r in n ? Rr(n, r, { enumerable: !0, configurable: !0, writable: !0, value: t }) : n[r] = t, On = (n, r) => {
    for (var t in r || (r = {}))
        qr.call(r, t) && Mn(n, t, r[t]);
    if (An)
        for (var t of An(r))
            zr.call(r, t) && Mn(n, t, r[t]);
    return n;
}, En = (n, r) => Vr(n, $r(r));
var rn = /* @__PURE__ */ ((n) => (n.First = "FIRST", n.Maximum = "MAXIMUM", n))(rn || {});
let L = null;
function Lr(n) {
    L = n;
}
Shopify = {
    main(n) {
        return L == null ? void 0 : L(n);
    }
};
var zn = "1.13.6", In = typeof self == "object" && self.self === self && self || typeof global == "object" && global.global === global && global || Function("return this")() || {}, X = Array.prototype, tn = Object.prototype, Nn = typeof Symbol != "undefined" ? Symbol.prototype : null, Ur = X.push, V = X.slice, D = tn.toString, Cr = tn.hasOwnProperty, Ln = typeof ArrayBuffer != "undefined", Wr = typeof DataView != "undefined", Jr = Array.isArray, Bn = Object.keys, Pn = Object.create, Sn = Ln && ArrayBuffer.isView, Xr = isNaN, Yr = isFinite, Un = !{ toString: null }.propertyIsEnumerable("toString"), Tn = [
    "valueOf",
    "isPrototypeOf",
    "toString",
    "propertyIsEnumerable",
    "hasOwnProperty",
    "toLocaleString"
], Gr = Math.pow(2, 53) - 1;
function m(n, r) {
    return r = r == null ? n.length - 1 : +r, function () {
        for (var t = Math.max(arguments.length - r, 0), e = Array(t), i = 0; i < t; i++)
            e[i] = arguments[i + r];
        switch (r) {
            case 0:
                return n.call(this, e);
            case 1:
                return n.call(this, arguments[0], e);
            case 2:
                return n.call(this, arguments[0], arguments[1], e);
        }
        var u = Array(r + 1);
        for (i = 0; i < r; i++)
            u[i] = arguments[i];
        return u[r] = e, n.apply(this, u);
    };
}
function I(n) {
    var r = typeof n;
    return r === "function" || r === "object" && !!n;
}
function Hr(n) {
    return n === null;
}
function Cn(n) {
    return n === void 0;
}
function Wn(n) {
    return n === !0 || n === !1 || D.call(n) === "[object Boolean]";
}
function Qr(n) {
    return !!(n && n.nodeType === 1);
}
function h(n) {
    var r = "[object " + n + "]";
    return function (t) {
        return D.call(t) === r;
    };
}
const en = h("String"), Jn = h("Number"), Zr = h("Date"), Kr = h("RegExp"), xr = h("Error"), Xn = h("Symbol"), Yn = h("ArrayBuffer");
var Gn = h("Function"), kr = In.document && In.document.childNodes;
typeof /./ != "function" && typeof Int8Array != "object" && typeof kr != "function" && (Gn = function (n) {
    return typeof n == "function" || !1;
});
const g = Gn, Hn = h("Object");
var Qn = Wr && Hn(new DataView(new ArrayBuffer(8))), un = typeof Map != "undefined" && Hn(/* @__PURE__ */ new Map()), br = h("DataView");
function jr(n) {
    return n != null && g(n.getInt8) && Yn(n.buffer);
}
const C = Qn ? jr : br, N = Jr || h("Array");
function M(n, r) {
    return n != null && Cr.call(n, r);
}
var Z = h("Arguments");
(function () {
    Z(arguments) || (Z = function (n) {
        return M(n, "callee");
    });
})();
const fn = Z;
function nt(n) {
    return !Xn(n) && Yr(n) && !isNaN(parseFloat(n));
}
function Zn(n) {
    return Jn(n) && Xr(n);
}
function Kn(n) {
    return function () {
        return n;
    };
}
function xn(n) {
    return function (r) {
        var t = n(r);
        return typeof t == "number" && t >= 0 && t <= Gr;
    };
}
function kn(n) {
    return function (r) {
        return r == null ? void 0 : r[n];
    };
}
const W = kn("byteLength"), rt = xn(W);
var tt = /\[object ((I|Ui)nt(8|16|32)|Float(32|64)|Uint8Clamped|Big(I|Ui)nt64)Array\]/;
function et(n) {
    return Sn ? Sn(n) && !C(n) : rt(n) && tt.test(D.call(n));
}
const bn = Ln ? et : Kn(!1), y = kn("length");
function ut(n) {
    for (var r = {}, t = n.length, e = 0; e < t; ++e)
        r[n[e]] = !0;
    return {
        contains: function (i) {
            return r[i] === !0;
        },
        push: function (i) {
            return r[i] = !0, n.push(i);
        }
    };
}
function jn(n, r) {
    r = ut(r);
    var t = Tn.length, e = n.constructor, i = g(e) && e.prototype || tn, u = "constructor";
    for (M(n, u) && !r.contains(u) && r.push(u); t--;)
        u = Tn[t], u in n && n[u] !== i[u] && !r.contains(u) && r.push(u);
}
function v(n) {
    if (!I(n))
        return [];
    if (Bn)
        return Bn(n);
    var r = [];
    for (var t in n)
        M(n, t) && r.push(t);
    return Un && jn(n, r), r;
}
function it(n) {
    if (n == null)
        return !0;
    var r = y(n);
    return typeof r == "number" && (N(n) || en(n) || fn(n)) ? r === 0 : y(v(n)) === 0;
}
function nr(n, r) {
    var t = v(r), e = t.length;
    if (n == null)
        return !e;
    for (var i = Object(n), u = 0; u < e; u++) {
        var f = t[u];
        if (r[f] !== i[f] || !(f in i))
            return !1;
    }
    return !0;
}
function o(n) {
    if (n instanceof o)
        return n;
    if (!(this instanceof o))
        return new o(n);
    this._wrapped = n;
}
o.VERSION = zn;
o.prototype.value = function () {
    return this._wrapped;
};
o.prototype.valueOf = o.prototype.toJSON = o.prototype.value;
o.prototype.toString = function () {
    return String(this._wrapped);
};
function Dn(n) {
    return new Uint8Array(
        n.buffer || n,
        n.byteOffset || 0,
        W(n)
    );
}
var Fn = "[object DataView]";
function K(n, r, t, e) {
    if (n === r)
        return n !== 0 || 1 / n === 1 / r;
    if (n == null || r == null)
        return !1;
    if (n !== n)
        return r !== r;
    var i = typeof n;
    return i !== "function" && i !== "object" && typeof r != "object" ? !1 : rr(n, r, t, e);
}
function rr(n, r, t, e) {
    n instanceof o && (n = n._wrapped), r instanceof o && (r = r._wrapped);
    var i = D.call(n);
    if (i !== D.call(r))
        return !1;
    if (Qn && i == "[object Object]" && C(n)) {
        if (!C(r))
            return !1;
        i = Fn;
    }
    switch (i) {
        case "[object RegExp]":
        case "[object String]":
            return "" + n == "" + r;
        case "[object Number]":
            return +n != +n ? +r != +r : +n == 0 ? 1 / +n === 1 / r : +n == +r;
        case "[object Date]":
        case "[object Boolean]":
            return +n == +r;
        case "[object Symbol]":
            return Nn.valueOf.call(n) === Nn.valueOf.call(r);
        case "[object ArrayBuffer]":
        case Fn:
            return rr(Dn(n), Dn(r), t, e);
    }
    var u = i === "[object Array]";
    if (!u && bn(n)) {
        var f = W(n);
        if (f !== W(r))
            return !1;
        if (n.buffer === r.buffer && n.byteOffset === r.byteOffset)
            return !0;
        u = !0;
    }
    if (!u) {
        if (typeof n != "object" || typeof r != "object")
            return !1;
        var l = n.constructor, c = r.constructor;
        if (l !== c && !(g(l) && l instanceof l && g(c) && c instanceof c) && "constructor" in n && "constructor" in r)
            return !1;
    }
    t = t || [], e = e || [];
    for (var a = t.length; a--;)
        if (t[a] === n)
            return e[a] === r;
    if (t.push(n), e.push(r), u) {
        if (a = n.length, a !== r.length)
            return !1;
        for (; a--;)
            if (!K(n[a], r[a], t, e))
                return !1;
    } else {
        var s = v(n), p;
        if (a = s.length, v(r).length !== a)
            return !1;
        for (; a--;)
            if (p = s[a], !(M(r, p) && K(n[p], r[p], t, e)))
                return !1;
    }
    return t.pop(), e.pop(), !0;
}
function ft(n, r) {
    return K(n, r);
}
function $(n) {
    if (!I(n))
        return [];
    var r = [];
    for (var t in n)
        r.push(t);
    return Un && jn(n, r), r;
}
function ln(n) {
    var r = y(n);
    return function (t) {
        if (t == null)
            return !1;
        var e = $(t);
        if (y(e))
            return !1;
        for (var i = 0; i < r; i++)
            if (!g(t[n[i]]))
                return !1;
        return n !== ur || !g(t[an]);
    };
}
var an = "forEach", tr = "has", cn = ["clear", "delete"], er = ["get", tr, "set"], lt = cn.concat(an, er), ur = cn.concat(er), at = ["add"].concat(cn, an, tr);
const ct = un ? ln(lt) : h("Map"), ot = un ? ln(ur) : h("WeakMap"), st = un ? ln(at) : h("Set"), vt = h("WeakSet");
function S(n) {
    for (var r = v(n), t = r.length, e = Array(t), i = 0; i < t; i++)
        e[i] = n[r[i]];
    return e;
}
function ht(n) {
    for (var r = v(n), t = r.length, e = Array(t), i = 0; i < t; i++)
        e[i] = [r[i], n[r[i]]];
    return e;
}
function ir(n) {
    for (var r = {}, t = v(n), e = 0, i = t.length; e < i; e++)
        r[n[t[e]]] = t[e];
    return r;
}
function x(n) {
    var r = [];
    for (var t in n)
        g(n[t]) && r.push(t);
    return r.sort();
}
function on(n, r) {
    return function (t) {
        var e = arguments.length;
        if (r && (t = Object(t)), e < 2 || t == null)
            return t;
        for (var i = 1; i < e; i++)
            for (var u = arguments[i], f = n(u), l = f.length, c = 0; c < l; c++) {
                var a = f[c];
                (!r || t[a] === void 0) && (t[a] = u[a]);
            }
        return t;
    };
}
const fr = on($), J = on(v), lr = on($, !0);
function pt() {
    return function () {
    };
}
function ar(n) {
    if (!I(n))
        return {};
    if (Pn)
        return Pn(n);
    var r = pt();
    r.prototype = n;
    var t = new r();
    return r.prototype = null, t;
}
function gt(n, r) {
    var t = ar(n);
    return r && J(t, r), t;
}
function mt(n) {
    return I(n) ? N(n) ? n.slice() : fr({}, n) : n;
}
function yt(n, r) {
    return r(n), n;
}
function cr(n) {
    return N(n) ? n : [n];
}
o.toPath = cr;
function q(n) {
    return o.toPath(n);
}
function sn(n, r) {
    for (var t = r.length, e = 0; e < t; e++) {
        if (n == null)
            return;
        n = n[r[e]];
    }
    return t ? n : void 0;
}
function or(n, r, t) {
    var e = sn(n, q(r));
    return Cn(e) ? t : e;
}
function dt(n, r) {
    r = q(r);
    for (var t = r.length, e = 0; e < t; e++) {
        var i = r[e];
        if (!M(n, i))
            return !1;
        n = n[i];
    }
    return !!t;
}
function vn(n) {
    return n;
}
function F(n) {
    return n = J({}, n), function (r) {
        return nr(r, n);
    };
}
function hn(n) {
    return n = q(n), function (r) {
        return sn(r, n);
    };
}
function z(n, r, t) {
    if (r === void 0)
        return n;
    switch (t == null ? 3 : t) {
        case 1:
            return function (e) {
                return n.call(r, e);
            };
        case 3:
            return function (e, i, u) {
                return n.call(r, e, i, u);
            };
        case 4:
            return function (e, i, u, f) {
                return n.call(r, e, i, u, f);
            };
    }
    return function () {
        return n.apply(r, arguments);
    };
}
function sr(n, r, t) {
    return n == null ? vn : g(n) ? z(n, r, t) : I(n) && !N(n) ? F(n) : hn(n);
}
function pn(n, r) {
    return sr(n, r, 1 / 0);
}
o.iteratee = pn;
function d(n, r, t) {
    return o.iteratee !== pn ? o.iteratee(n, r) : sr(n, r, t);
}
function wt(n, r, t) {
    r = d(r, t);
    for (var e = v(n), i = e.length, u = {}, f = 0; f < i; f++) {
        var l = e[f];
        u[l] = r(n[l], l, n);
    }
    return u;
}
function vr() {
}
function _t(n) {
    return n == null ? vr : function (r) {
        return or(n, r);
    };
}
function At(n, r, t) {
    var e = Array(Math.max(0, n));
    r = z(r, t, 1);
    for (var i = 0; i < n; i++)
        e[i] = r(i);
    return e;
}
function k(n, r) {
    return r == null && (r = n, n = 0), n + Math.floor(Math.random() * (r - n + 1));
}
const R = Date.now || function () {
    return new Date().getTime();
};
function hr(n) {
    var r = function (u) {
        return n[u];
    }, t = "(?:" + v(n).join("|") + ")", e = RegExp(t), i = RegExp(t, "g");
    return function (u) {
        return u = u == null ? "" : "" + u, e.test(u) ? u.replace(i, r) : u;
    };
}
const pr = {
    "&": "&amp;",
    "<": "&lt;",
    ">": "&gt;",
    '"': "&quot;",
    "'": "&#x27;",
    "`": "&#x60;"
}, Mt = hr(pr), Ot = ir(pr), Et = hr(Ot), It = o.templateSettings = {
    evaluate: /<%([\s\S]+?)%>/g,
    interpolate: /<%=([\s\S]+?)%>/g,
    escape: /<%-([\s\S]+?)%>/g
};
var G = /(.)^/, Nt = {
    "'": "'",
    "\\": "\\",
    "\r": "r",
    "\n": "n",
    "\u2028": "u2028",
    "\u2029": "u2029"
}, Bt = /\\|'|\r|\n|\u2028|\u2029/g;
function Pt(n) {
    return "\\" + Nt[n];
}
var St = /^\s*(\w|\$)+\s*$/;
function Tt(n, r, t) {
    !r && t && (r = t), r = lr({}, r, o.templateSettings);
    var e = RegExp([
        (r.escape || G).source,
        (r.interpolate || G).source,
        (r.evaluate || G).source
    ].join("|") + "|$", "g"), i = 0, u = "__p+='";
    n.replace(e, function (a, s, p, wn, _n) {
        return u += n.slice(i, _n).replace(Bt, Pt), i = _n + a.length, s ? u += `'+
((__t=(` + s + `))==null?'':_.escape(__t))+
'` : p ? u += `'+
((__t=(` + p + `))==null?'':__t)+
'` : wn && (u += `';
` + wn + `
__p+='`), a;
    }), u += `';
`;
    var f = r.variable;
    if (f) {
        if (!St.test(f))
            throw new Error(
                "variable is not a bare identifier: " + f
            );
    } else
        u = `with(obj||{}){
` + u + `}
`, f = "obj";
    u = `var __t,__p='',__j=Array.prototype.join,print=function(){__p+=__j.call(arguments,'');};
` + u + `return __p;
`;
    var l;
    try {
        l = new Function(f, "_", u);
    } catch (a) {
        throw a.source = u, a;
    }
    var c = function (a) {
        return l.call(this, a, o);
    };
    return c.source = "function(" + f + `){
` + u + "}", c;
}
function Dt(n, r, t) {
    r = q(r);
    var e = r.length;
    if (!e)
        return g(t) ? t.call(n) : t;
    for (var i = 0; i < e; i++) {
        var u = n == null ? void 0 : n[r[i]];
        u === void 0 && (u = t, i = e), n = g(u) ? u.call(n) : u;
    }
    return n;
}
var Ft = 0;
function Rt(n) {
    var r = ++Ft + "";
    return n ? n + r : r;
}
function Vt(n) {
    var r = o(n);
    return r._chain = !0, r;
}
function gr(n, r, t, e, i) {
    if (!(e instanceof r))
        return n.apply(t, i);
    var u = ar(n.prototype), f = n.apply(u, i);
    return I(f) ? f : u;
}
var T = m(function (n, r) {
    var t = T.placeholder, e = function () {
        for (var i = 0, u = r.length, f = Array(u), l = 0; l < u; l++)
            f[l] = r[l] === t ? arguments[i++] : r[l];
        for (; i < arguments.length;)
            f.push(arguments[i++]);
        return gr(n, e, this, this, f);
    };
    return e;
});
T.placeholder = o;
const mr = m(function (n, r, t) {
    if (!g(n))
        throw new TypeError("Bind must be called on a function");
    var e = m(function (i) {
        return gr(n, e, r, this, t.concat(i));
    });
    return e;
}), w = xn(y);
function B(n, r, t, e) {
    if (e = e || [], !r && r !== 0)
        r = 1 / 0;
    else if (r <= 0)
        return e.concat(n);
    for (var i = e.length, u = 0, f = y(n); u < f; u++) {
        var l = n[u];
        if (w(l) && (N(l) || fn(l)))
            if (r > 1)
                B(l, r - 1, t, e), i = e.length;
            else
                for (var c = 0, a = l.length; c < a;)
                    e[i++] = l[c++];
        else
            t || (e[i++] = l);
    }
    return e;
}
const $t = m(function (n, r) {
    r = B(r, !1, !1);
    var t = r.length;
    if (t < 1)
        throw new Error("bindAll must be passed function names");
    for (; t--;) {
        var e = r[t];
        n[e] = mr(n[e], n);
    }
    return n;
});
function qt(n, r) {
    var t = function (e) {
        var i = t.cache, u = "" + (r ? r.apply(this, arguments) : e);
        return M(i, u) || (i[u] = n.apply(this, arguments)), i[u];
    };
    return t.cache = {}, t;
}
const yr = m(function (n, r, t) {
    return setTimeout(function () {
        return n.apply(null, t);
    }, r);
}), zt = T(yr, o, 1);
function Lt(n, r, t) {
    var e, i, u, f, l = 0;
    t || (t = {});
    var c = function () {
        l = t.leading === !1 ? 0 : R(), e = null, f = n.apply(i, u), e || (i = u = null);
    }, a = function () {
        var s = R();
        !l && t.leading === !1 && (l = s);
        var p = r - (s - l);
        return i = this, u = arguments, p <= 0 || p > r ? (e && (clearTimeout(e), e = null), l = s, f = n.apply(i, u), e || (i = u = null)) : !e && t.trailing !== !1 && (e = setTimeout(c, p)), f;
    };
    return a.cancel = function () {
        clearTimeout(e), l = 0, e = i = u = null;
    }, a;
}
function Ut(n, r, t) {
    var e, i, u, f, l, c = function () {
        var s = R() - i;
        r > s ? e = setTimeout(c, r - s) : (e = null, t || (f = n.apply(l, u)), e || (u = l = null));
    }, a = m(function (s) {
        return l = this, u = s, i = R(), e || (e = setTimeout(c, r), t && (f = n.apply(l, u))), f;
    });
    return a.cancel = function () {
        clearTimeout(e), e = u = l = null;
    }, a;
}
function Ct(n, r) {
    return T(r, n);
}
function gn(n) {
    return function () {
        return !n.apply(this, arguments);
    };
}
function Wt() {
    var n = arguments, r = n.length - 1;
    return function () {
        for (var t = r, e = n[r].apply(this, arguments); t--;)
            e = n[t].call(this, e);
        return e;
    };
}
function Jt(n, r) {
    return function () {
        if (--n < 1)
            return r.apply(this, arguments);
    };
}
function dr(n, r) {
    var t;
    return function () {
        return --n > 0 && (t = r.apply(this, arguments)), n <= 1 && (r = null), t;
    };
}
const Xt = T(dr, 2);
function wr(n, r, t) {
    r = d(r, t);
    for (var e = v(n), i, u = 0, f = e.length; u < f; u++)
        if (i = e[u], r(n[i], i, n))
            return i;
}
function _r(n) {
    return function (r, t, e) {
        t = d(t, e);
        for (var i = y(r), u = n > 0 ? 0 : i - 1; u >= 0 && u < i; u += n)
            if (t(r[u], u, r))
                return u;
        return -1;
    };
}
const mn = _r(1), Ar = _r(-1);
function Mr(n, r, t, e) {
    t = d(t, e, 1);
    for (var i = t(r), u = 0, f = y(n); u < f;) {
        var l = Math.floor((u + f) / 2);
        t(n[l]) < i ? u = l + 1 : f = l;
    }
    return u;
}
function Or(n, r, t) {
    return function (e, i, u) {
        var f = 0, l = y(e);
        if (typeof u == "number")
            n > 0 ? f = u >= 0 ? u : Math.max(u + l, f) : l = u >= 0 ? Math.min(u + 1, l) : u + l + 1;
        else if (t && u && l)
            return u = t(e, i), e[u] === i ? u : -1;
        if (i !== i)
            return u = r(V.call(e, f, l), Zn), u >= 0 ? u + f : -1;
        for (u = n > 0 ? f : l - 1; u >= 0 && u < l; u += n)
            if (e[u] === i)
                return u;
        return -1;
    };
}
const Er = Or(1, mn, Mr), Yt = Or(-1, Ar);
function b(n, r, t) {
    var e = w(n) ? mn : wr, i = e(n, r, t);
    if (i !== void 0 && i !== -1)
        return n[i];
}
function Gt(n, r) {
    return b(n, F(r));
}
function A(n, r, t) {
    r = z(r, t);
    var e, i;
    if (w(n))
        for (e = 0, i = n.length; e < i; e++)
            r(n[e], e, n);
    else {
        var u = v(n);
        for (e = 0, i = u.length; e < i; e++)
            r(n[u[e]], u[e], n);
    }
    return n;
}
function E(n, r, t) {
    r = d(r, t);
    for (var e = !w(n) && v(n), i = (e || n).length, u = Array(i), f = 0; f < i; f++) {
        var l = e ? e[f] : f;
        u[f] = r(n[l], l, n);
    }
    return u;
}
function Ir(n) {
    var r = function (t, e, i, u) {
        var f = !w(t) && v(t), l = (f || t).length, c = n > 0 ? 0 : l - 1;
        for (u || (i = t[f ? f[c] : c], c += n); c >= 0 && c < l; c += n) {
            var a = f ? f[c] : c;
            i = e(i, t[a], a, t);
        }
        return i;
    };
    return function (t, e, i, u) {
        var f = arguments.length >= 3;
        return r(t, z(e, u, 4), i, f);
    };
}
const H = Ir(1), Rn = Ir(-1);
function P(n, r, t) {
    var e = [];
    return r = d(r, t), A(n, function (i, u, f) {
        r(i, u, f) && e.push(i);
    }), e;
}
function Ht(n, r, t) {
    return P(n, gn(d(r)), t);
}
function Vn(n, r, t) {
    r = d(r, t);
    for (var e = !w(n) && v(n), i = (e || n).length, u = 0; u < i; u++) {
        var f = e ? e[u] : u;
        if (!r(n[f], f, n))
            return !1;
    }
    return !0;
}
function $n(n, r, t) {
    r = d(r, t);
    for (var e = !w(n) && v(n), i = (e || n).length, u = 0; u < i; u++) {
        var f = e ? e[u] : u;
        if (r(n[f], f, n))
            return !0;
    }
    return !1;
}
function _(n, r, t, e) {
    return w(n) || (n = S(n)), (typeof t != "number" || e) && (t = 0), Er(n, r, t) >= 0;
}
const Qt = m(function (n, r, t) {
    var e, i;
    return g(r) ? i = r : (r = q(r), e = r.slice(0, -1), r = r[r.length - 1]), E(n, function (u) {
        var f = i;
        if (!f) {
            if (e && e.length && (u = sn(u, e)), u == null)
                return;
            f = u[r];
        }
        return f == null ? f : f.apply(u, t);
    });
});
function yn(n, r) {
    return E(n, hn(r));
}
function Zt(n, r) {
    return P(n, F(r));
}
function Nr(n, r, t) {
    var e = -1 / 0, i = -1 / 0, u, f;
    if (r == null || typeof r == "number" && typeof n[0] != "object" && n != null) {
        n = w(n) ? n : S(n);
        for (var l = 0, c = n.length; l < c; l++)
            u = n[l], u != null && u > e && (e = u);
    } else
        r = d(r, t), A(n, function (a, s, p) {
            f = r(a, s, p), (f > i || f === -1 / 0 && e === -1 / 0) && (e = a, i = f);
        });
    return e;
}
function Kt(n, r, t) {
    var e = 1 / 0, i = 1 / 0, u, f;
    if (r == null || typeof r == "number" && typeof n[0] != "object" && n != null) {
        n = w(n) ? n : S(n);
        for (var l = 0, c = n.length; l < c; l++)
            u = n[l], u != null && u < e && (e = u);
    } else
        r = d(r, t), A(n, function (a, s, p) {
            f = r(a, s, p), (f < i || f === 1 / 0 && e === 1 / 0) && (e = a, i = f);
        });
    return e;
}
var xt = /[^\ud800-\udfff]|[\ud800-\udbff][\udc00-\udfff]|[\ud800-\udfff]/g;
function Br(n) {
    return n ? N(n) ? V.call(n) : en(n) ? n.match(xt) : w(n) ? E(n, vn) : S(n) : [];
}
function Pr(n, r, t) {
    if (r == null || t)
        return w(n) || (n = S(n)), n[k(n.length - 1)];
    var e = Br(n), i = y(e);
    r = Math.max(Math.min(r, i), 0);
    for (var u = i - 1, f = 0; f < r; f++) {
        var l = k(f, u), c = e[f];
        e[f] = e[l], e[l] = c;
    }
    return e.slice(0, r);
}
function kt(n) {
    return Pr(n, 1 / 0);
}
function bt(n, r, t) {
    var e = 0;
    return r = d(r, t), yn(E(n, function (i, u, f) {
        return {
            value: i,
            index: e++,
            criteria: r(i, u, f)
        };
    }).sort(function (i, u) {
        var f = i.criteria, l = u.criteria;
        if (f !== l) {
            if (f > l || f === void 0)
                return 1;
            if (f < l || l === void 0)
                return -1;
        }
        return i.index - u.index;
    }), "value");
}
function Y(n, r) {
    return function (t, e, i) {
        var u = r ? [[], []] : {};
        return e = d(e, i), A(t, function (f, l) {
            var c = e(f, l, t);
            n(u, f, c);
        }), u;
    };
}
const jt = Y(function (n, r, t) {
    M(n, t) ? n[t].push(r) : n[t] = [r];
}), ne = Y(function (n, r, t) {
    n[t] = r;
}), re = Y(function (n, r, t) {
    M(n, t) ? n[t]++ : n[t] = 1;
}), te = Y(function (n, r, t) {
    n[t ? 0 : 1].push(r);
}, !0);
function ee(n) {
    return n == null ? 0 : w(n) ? n.length : v(n).length;
}
function ue(n, r, t) {
    return r in t;
}
const Sr = m(function (n, r) {
    var t = {}, e = r[0];
    if (n == null)
        return t;
    g(e) ? (r.length > 1 && (e = z(e, r[1])), r = $(n)) : (e = ue, r = B(r, !1, !1), n = Object(n));
    for (var i = 0, u = r.length; i < u; i++) {
        var f = r[i], l = n[f];
        e(l, f, n) && (t[f] = l);
    }
    return t;
}), ie = m(function (n, r) {
    var t = r[0], e;
    return g(t) ? (t = gn(t), r.length > 1 && (e = r[1])) : (r = E(B(r, !1, !1), String), t = function (i, u) {
        return !_(r, u);
    }), Sr(n, t, e);
});
function Tr(n, r, t) {
    return V.call(n, 0, Math.max(0, n.length - (r == null || t ? 1 : r)));
}
function Q(n, r, t) {
    return n == null || n.length < 1 ? r == null || t ? void 0 : [] : r == null || t ? n[0] : Tr(n, n.length - r);
}
function U(n, r, t) {
    return V.call(n, r == null || t ? 1 : r);
}
function fe(n, r, t) {
    return n == null || n.length < 1 ? r == null || t ? void 0 : [] : r == null || t ? n[n.length - 1] : U(n, Math.max(0, n.length - r));
}
function le(n) {
    return P(n, Boolean);
}
function ae(n, r) {
    return B(n, r, !1);
}
const Dr = m(function (n, r) {
    return r = B(r, !0, !0), P(n, function (t) {
        return !_(r, t);
    });
}), ce = m(function (n, r) {
    return Dr(n, r);
});
function j(n, r, t, e) {
    Wn(r) || (e = t, t = r, r = !1), t != null && (t = d(t, e));
    for (var i = [], u = [], f = 0, l = y(n); f < l; f++) {
        var c = n[f], a = t ? t(c, f, n) : c;
        r && !t ? ((!f || u !== a) && i.push(c), u = a) : t ? _(u, a) || (u.push(a), i.push(c)) : _(i, c) || i.push(c);
    }
    return i;
}
const oe = m(function (n) {
    return j(B(n, !0, !0));
});
function se(n) {
    for (var r = [], t = arguments.length, e = 0, i = y(n); e < i; e++) {
        var u = n[e];
        if (!_(r, u)) {
            var f;
            for (f = 1; f < t && _(arguments[f], u); f++)
                ;
            f === t && r.push(u);
        }
    }
    return r;
}
function nn(n) {
    for (var r = n && Nr(n, y).length || 0, t = Array(r), e = 0; e < r; e++)
        t[e] = yn(n, e);
    return t;
}
const ve = m(nn);
function he(n, r) {
    for (var t = {}, e = 0, i = y(n); e < i; e++)
        r ? t[n[e]] = r[e] : t[n[e][0]] = n[e][1];
    return t;
}
function pe(n, r, t) {
    r == null && (r = n || 0, n = 0), t || (t = r < n ? -1 : 1);
    for (var e = Math.max(Math.ceil((r - n) / t), 0), i = Array(e), u = 0; u < e; u++, n += t)
        i[u] = n;
    return i;
}
function ge(n, r) {
    if (r == null || r < 1)
        return [];
    for (var t = [], e = 0, i = n.length; e < i;)
        t.push(V.call(n, e, e += r));
    return t;
}
function dn(n, r) {
    return n._chain ? o(r).chain() : r;
}
function Fr(n) {
    return A(x(n), function (r) {
        var t = o[r] = n[r];
        o.prototype[r] = function () {
            var e = [this._wrapped];
            return Ur.apply(e, arguments), dn(this, t.apply(o, e));
        };
    }), o;
}
A(["pop", "push", "reverse", "shift", "sort", "splice", "unshift"], function (n) {
    var r = X[n];
    o.prototype[n] = function () {
        var t = this._wrapped;
        return t != null && (r.apply(t, arguments), (n === "shift" || n === "splice") && t.length === 0 && delete t[0]), dn(this, t);
    };
});
A(["concat", "join", "slice"], function (n) {
    var r = X[n];
    o.prototype[n] = function () {
        var t = this._wrapped;
        return t != null && (t = r.apply(t, arguments)), dn(this, t);
    };
});
const me = /* @__PURE__ */ Object.freeze(/* @__PURE__ */ Object.defineProperty({
    __proto__: null,
    VERSION: zn,
    restArguments: m,
    isObject: I,
    isNull: Hr,
    isUndefined: Cn,
    isBoolean: Wn,
    isElement: Qr,
    isString: en,
    isNumber: Jn,
    isDate: Zr,
    isRegExp: Kr,
    isError: xr,
    isSymbol: Xn,
    isArrayBuffer: Yn,
    isDataView: C,
    isArray: N,
    isFunction: g,
    isArguments: fn,
    isFinite: nt,
    isNaN: Zn,
    isTypedArray: bn,
    isEmpty: it,
    isMatch: nr,
    isEqual: ft,
    isMap: ct,
    isWeakMap: ot,
    isSet: st,
    isWeakSet: vt,
    keys: v,
    allKeys: $,
    values: S,
    pairs: ht,
    invert: ir,
    functions: x,
    methods: x,
    extend: fr,
    extendOwn: J,
    assign: J,
    defaults: lr,
    create: gt,
    clone: mt,
    tap: yt,
    get: or,
    has: dt,
    mapObject: wt,
    identity: vn,
    constant: Kn,
    noop: vr,
    toPath: cr,
    property: hn,
    propertyOf: _t,
    matcher: F,
    matches: F,
    times: At,
    random: k,
    now: R,
    escape: Mt,
    unescape: Et,
    templateSettings: It,
    template: Tt,
    result: Dt,
    uniqueId: Rt,
    chain: Vt,
    iteratee: pn,
    partial: T,
    bind: mr,
    bindAll: $t,
    memoize: qt,
    delay: yr,
    defer: zt,
    throttle: Lt,
    debounce: Ut,
    wrap: Ct,
    negate: gn,
    compose: Wt,
    after: Jt,
    before: dr,
    once: Xt,
    findKey: wr,
    findIndex: mn,
    findLastIndex: Ar,
    sortedIndex: Mr,
    indexOf: Er,
    lastIndexOf: Yt,
    find: b,
    detect: b,
    findWhere: Gt,
    each: A,
    forEach: A,
    map: E,
    collect: E,
    reduce: H,
    foldl: H,
    inject: H,
    reduceRight: Rn,
    foldr: Rn,
    filter: P,
    select: P,
    reject: Ht,
    every: Vn,
    all: Vn,
    some: $n,
    any: $n,
    contains: _,
    includes: _,
    include: _,
    invoke: Qt,
    pluck: yn,
    where: Zt,
    max: Nr,
    min: Kt,
    shuffle: kt,
    sample: Pr,
    sortBy: bt,
    groupBy: jt,
    indexBy: ne,
    countBy: re,
    partition: te,
    toArray: Br,
    size: ee,
    pick: Sr,
    omit: ie,
    first: Q,
    head: Q,
    take: Q,
    initial: Tr,
    last: fe,
    rest: U,
    tail: U,
    drop: U,
    compact: le,
    flatten: ae,
    without: ce,
    uniq: j,
    unique: j,
    union: oe,
    intersection: se,
    difference: Dr,
    unzip: nn,
    transpose: nn,
    zip: ve,
    object: he,
    range: pe,
    chunk: ge,
    mixin: Fr,
    default: o
}, Symbol.toStringTag, { value: "Module" }));
var O = Fr(me);
O._ = O;
const qn = {
    discountApplicationStrategy: rn.First,
    discounts: []
};
Lr((n) => {
    const r = JSON.parse(
        O.get(n, ["discountNode", "metafield", "value"], "{}")
    );
    if (!JSON.parse(
        O.get(n, ["cart", "buyerIdentity", "customer", "metafield", "value"], "{}")
    ))
        return qn;
    const e = O.chain(n.cart.lines).sortBy((u) => u.quantity).map((u) => En(On({}, u), { id: O.escape(u.id) })).value();
    return O.reduce(e, (u, f) => u + f.quantity, 0) < 0 ? qn : {
        discountApplicationStrategy: rn.Maximum,
        discounts: [
            {
                message: "VIP Discount",
                targets: [
                    {
                        productVariant: {
                            id: e[0].id
                        }
                    }
                ],
                value: {
                    percentage: {
                        value: r.discountPercentage
                    }
                }
            }
        ]
    };
});