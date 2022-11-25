var zr = Object.defineProperty, Lr = Object.defineProperties;
var Ur = Object.getOwnPropertyDescriptors;
var Tn = Object.getOwnPropertySymbols;
var Wr = Object.prototype.hasOwnProperty, Jr = Object.prototype.propertyIsEnumerable;
var Bn = (n, r, t) => r in n ? zr(n, r, { enumerable: !0, configurable: !0, writable: !0, value: t }) : n[r] = t, Nn = (n, r) => {
  for (var t in r || (r = {}))
    Wr.call(r, t) && Bn(n, t, r[t]);
  if (Tn)
    for (var t of Tn(r))
      Jr.call(r, t) && Bn(n, t, r[t]);
  return n;
}, Dn = (n, r) => Lr(n, Ur(r));
var ln = /* @__PURE__ */ ((n) => (n.First = "FIRST", n.Maximum = "MAXIMUM", n))(ln || {});
let Y = null;
function Xr(n) {
  Y = n;
}
Shopify = {
  main(n) {
    return Y == null ? void 0 : Y(n);
  }
};
var Jn = "1.13.6", Fn = typeof self == "object" && self.self === self && self || typeof global == "object" && global.global === global && global || Function("return this")() || {}, x = Array.prototype, on = Object.prototype, Sn = typeof Symbol != "undefined" ? Symbol.prototype : null, Yr = x.push, z = x.slice, R = on.toString, Gr = on.hasOwnProperty, Xn = typeof ArrayBuffer != "undefined", Hr = typeof DataView != "undefined", Qr = Array.isArray, Pn = Object.keys, Cn = Object.create, Vn = Xn && ArrayBuffer.isView, Zr = isNaN, xr = isFinite, Yn = !{ toString: null }.propertyIsEnumerable("toString"), Rn = [
  "valueOf",
  "isPrototypeOf",
  "toString",
  "propertyIsEnumerable",
  "hasOwnProperty",
  "toLocaleString"
], Kr = Math.pow(2, 53) - 1;
function A(n, r) {
  return r = r == null ? n.length - 1 : +r, function() {
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
function D(n) {
  var r = typeof n;
  return r === "function" || r === "object" && !!n;
}
function kr(n) {
  return n === null;
}
function Gn(n) {
  return n === void 0;
}
function Hn(n) {
  return n === !0 || n === !1 || R.call(n) === "[object Boolean]";
}
function jr(n) {
  return !!(n && n.nodeType === 1);
}
function d(n) {
  var r = "[object " + n + "]";
  return function(t) {
    return R.call(t) === r;
  };
}
const cn = d("String"), Qn = d("Number"), br = d("Date"), nt = d("RegExp"), rt = d("Error"), Zn = d("Symbol"), xn = d("ArrayBuffer");
var Kn = d("Function"), tt = Fn.document && Fn.document.childNodes;
typeof /./ != "function" && typeof Int8Array != "object" && typeof tt != "function" && (Kn = function(n) {
  return typeof n == "function" || !1;
});
const w = Kn, kn = d("Object");
var jn = Hr && kn(new DataView(new ArrayBuffer(8))), sn = typeof Map != "undefined" && kn(/* @__PURE__ */ new Map()), et = d("DataView");
function ut(n) {
  return n != null && w(n.getInt8) && xn(n.buffer);
}
const H = jn ? ut : et, F = Qr || d("Array");
function T(n, r) {
  return n != null && Gr.call(n, r);
}
var nn = d("Arguments");
(function() {
  nn(arguments) || (nn = function(n) {
    return T(n, "callee");
  });
})();
const vn = nn;
function it(n) {
  return !Zn(n) && xr(n) && !isNaN(parseFloat(n));
}
function bn(n) {
  return Qn(n) && Zr(n);
}
function nr(n) {
  return function() {
    return n;
  };
}
function rr(n) {
  return function(r) {
    var t = n(r);
    return typeof t == "number" && t >= 0 && t <= Kr;
  };
}
function tr(n) {
  return function(r) {
    return r == null ? void 0 : r[n];
  };
}
const Q = tr("byteLength"), ft = rr(Q);
var at = /\[object ((I|Ui)nt(8|16|32)|Float(32|64)|Uint8Clamped|Big(I|Ui)nt64)Array\]/;
function lt(n) {
  return Vn ? Vn(n) && !H(n) : ft(n) && at.test(R.call(n));
}
const er = Xn ? lt : nr(!1), _ = tr("length");
function ot(n) {
  for (var r = {}, t = n.length, e = 0; e < t; ++e)
    r[n[e]] = !0;
  return {
    contains: function(i) {
      return r[i] === !0;
    },
    push: function(i) {
      return r[i] = !0, n.push(i);
    }
  };
}
function ur(n, r) {
  r = ot(r);
  var t = Rn.length, e = n.constructor, i = w(e) && e.prototype || on, u = "constructor";
  for (T(n, u) && !r.contains(u) && r.push(u); t--; )
    u = Rn[t], u in n && n[u] !== i[u] && !r.contains(u) && r.push(u);
}
function y(n) {
  if (!D(n))
    return [];
  if (Pn)
    return Pn(n);
  var r = [];
  for (var t in n)
    T(n, t) && r.push(t);
  return Yn && ur(n, r), r;
}
function ct(n) {
  if (n == null)
    return !0;
  var r = _(n);
  return typeof r == "number" && (F(n) || cn(n) || vn(n)) ? r === 0 : _(y(n)) === 0;
}
function ir(n, r) {
  var t = y(r), e = t.length;
  if (n == null)
    return !e;
  for (var i = Object(n), u = 0; u < e; u++) {
    var f = t[u];
    if (r[f] !== i[f] || !(f in i))
      return !1;
  }
  return !0;
}
function v(n) {
  if (n instanceof v)
    return n;
  if (!(this instanceof v))
    return new v(n);
  this._wrapped = n;
}
v.VERSION = Jn;
v.prototype.value = function() {
  return this._wrapped;
};
v.prototype.valueOf = v.prototype.toJSON = v.prototype.value;
v.prototype.toString = function() {
  return String(this._wrapped);
};
function $n(n) {
  return new Uint8Array(
    n.buffer || n,
    n.byteOffset || 0,
    Q(n)
  );
}
var qn = "[object DataView]";
function rn(n, r, t, e) {
  if (n === r)
    return n !== 0 || 1 / n === 1 / r;
  if (n == null || r == null)
    return !1;
  if (n !== n)
    return r !== r;
  var i = typeof n;
  return i !== "function" && i !== "object" && typeof r != "object" ? !1 : fr(n, r, t, e);
}
function fr(n, r, t, e) {
  n instanceof v && (n = n._wrapped), r instanceof v && (r = r._wrapped);
  var i = R.call(n);
  if (i !== R.call(r))
    return !1;
  if (jn && i == "[object Object]" && H(n)) {
    if (!H(r))
      return !1;
    i = qn;
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
      return Sn.valueOf.call(n) === Sn.valueOf.call(r);
    case "[object ArrayBuffer]":
    case qn:
      return fr($n(n), $n(r), t, e);
  }
  var u = i === "[object Array]";
  if (!u && er(n)) {
    var f = Q(n);
    if (f !== Q(r))
      return !1;
    if (n.buffer === r.buffer && n.byteOffset === r.byteOffset)
      return !0;
    u = !0;
  }
  if (!u) {
    if (typeof n != "object" || typeof r != "object")
      return !1;
    var a = n.constructor, o = r.constructor;
    if (a !== o && !(w(a) && a instanceof a && w(o) && o instanceof o) && "constructor" in n && "constructor" in r)
      return !1;
  }
  t = t || [], e = e || [];
  for (var l = t.length; l--; )
    if (t[l] === n)
      return e[l] === r;
  if (t.push(n), e.push(r), u) {
    if (l = n.length, l !== r.length)
      return !1;
    for (; l--; )
      if (!rn(n[l], r[l], t, e))
        return !1;
  } else {
    var p = y(n), m;
    if (l = p.length, y(r).length !== l)
      return !1;
    for (; l--; )
      if (m = p[l], !(T(r, m) && rn(n[m], r[m], t, e)))
        return !1;
  }
  return t.pop(), e.pop(), !0;
}
function st(n, r) {
  return rn(n, r);
}
function L(n) {
  if (!D(n))
    return [];
  var r = [];
  for (var t in n)
    r.push(t);
  return Yn && ur(n, r), r;
}
function hn(n) {
  var r = _(n);
  return function(t) {
    if (t == null)
      return !1;
    var e = L(t);
    if (_(e))
      return !1;
    for (var i = 0; i < r; i++)
      if (!w(t[n[i]]))
        return !1;
    return n !== or || !w(t[pn]);
  };
}
var pn = "forEach", ar = "has", gn = ["clear", "delete"], lr = ["get", ar, "set"], vt = gn.concat(pn, lr), or = gn.concat(lr), ht = ["add"].concat(gn, pn, ar);
const pt = sn ? hn(vt) : d("Map"), gt = sn ? hn(or) : d("WeakMap"), yt = sn ? hn(ht) : d("Set"), mt = d("WeakSet");
function C(n) {
  for (var r = y(n), t = r.length, e = Array(t), i = 0; i < t; i++)
    e[i] = n[r[i]];
  return e;
}
function dt(n) {
  for (var r = y(n), t = r.length, e = Array(t), i = 0; i < t; i++)
    e[i] = [r[i], n[r[i]]];
  return e;
}
function cr(n) {
  for (var r = {}, t = y(n), e = 0, i = t.length; e < i; e++)
    r[n[t[e]]] = t[e];
  return r;
}
function tn(n) {
  var r = [];
  for (var t in n)
    w(n[t]) && r.push(t);
  return r.sort();
}
function yn(n, r) {
  return function(t) {
    var e = arguments.length;
    if (r && (t = Object(t)), e < 2 || t == null)
      return t;
    for (var i = 1; i < e; i++)
      for (var u = arguments[i], f = n(u), a = f.length, o = 0; o < a; o++) {
        var l = f[o];
        (!r || t[l] === void 0) && (t[l] = u[l]);
      }
    return t;
  };
}
const sr = yn(L), Z = yn(y), vr = yn(L, !0);
function wt() {
  return function() {
  };
}
function hr(n) {
  if (!D(n))
    return {};
  if (Cn)
    return Cn(n);
  var r = wt();
  r.prototype = n;
  var t = new r();
  return r.prototype = null, t;
}
function At(n, r) {
  var t = hr(n);
  return r && Z(t, r), t;
}
function _t(n) {
  return D(n) ? F(n) ? n.slice() : sr({}, n) : n;
}
function Mt(n, r) {
  return r(n), n;
}
function pr(n) {
  return F(n) ? n : [n];
}
v.toPath = pr;
function U(n) {
  return v.toPath(n);
}
function mn(n, r) {
  for (var t = r.length, e = 0; e < t; e++) {
    if (n == null)
      return;
    n = n[r[e]];
  }
  return t ? n : void 0;
}
function gr(n, r, t) {
  var e = mn(n, U(r));
  return Gn(e) ? t : e;
}
function Et(n, r) {
  r = U(r);
  for (var t = r.length, e = 0; e < t; e++) {
    var i = r[e];
    if (!T(n, i))
      return !1;
    n = n[i];
  }
  return !!t;
}
function dn(n) {
  return n;
}
function $(n) {
  return n = Z({}, n), function(r) {
    return ir(r, n);
  };
}
function wn(n) {
  return n = U(n), function(r) {
    return mn(r, n);
  };
}
function W(n, r, t) {
  if (r === void 0)
    return n;
  switch (t == null ? 3 : t) {
    case 1:
      return function(e) {
        return n.call(r, e);
      };
    case 3:
      return function(e, i, u) {
        return n.call(r, e, i, u);
      };
    case 4:
      return function(e, i, u, f) {
        return n.call(r, e, i, u, f);
      };
  }
  return function() {
    return n.apply(r, arguments);
  };
}
function yr(n, r, t) {
  return n == null ? dn : w(n) ? W(n, r, t) : D(n) && !F(n) ? $(n) : wn(n);
}
function An(n, r) {
  return yr(n, r, 1 / 0);
}
v.iteratee = An;
function M(n, r, t) {
  return v.iteratee !== An ? v.iteratee(n, r) : yr(n, r, t);
}
function Ot(n, r, t) {
  r = M(r, t);
  for (var e = y(n), i = e.length, u = {}, f = 0; f < i; f++) {
    var a = e[f];
    u[a] = r(n[a], a, n);
  }
  return u;
}
function mr() {
}
function It(n) {
  return n == null ? mr : function(r) {
    return gr(n, r);
  };
}
function Tt(n, r, t) {
  var e = Array(Math.max(0, n));
  r = W(r, t, 1);
  for (var i = 0; i < n; i++)
    e[i] = r(i);
  return e;
}
function en(n, r) {
  return r == null && (r = n, n = 0), n + Math.floor(Math.random() * (r - n + 1));
}
const q = Date.now || function() {
  return new Date().getTime();
};
function dr(n) {
  var r = function(u) {
    return n[u];
  }, t = "(?:" + y(n).join("|") + ")", e = RegExp(t), i = RegExp(t, "g");
  return function(u) {
    return u = u == null ? "" : "" + u, e.test(u) ? u.replace(i, r) : u;
  };
}
const wr = {
  "&": "&amp;",
  "<": "&lt;",
  ">": "&gt;",
  '"': "&quot;",
  "'": "&#x27;",
  "`": "&#x60;"
}, Bt = dr(wr), Nt = cr(wr), Dt = dr(Nt), Ft = v.templateSettings = {
  evaluate: /<%([\s\S]+?)%>/g,
  interpolate: /<%=([\s\S]+?)%>/g,
  escape: /<%-([\s\S]+?)%>/g
};
var k = /(.)^/, St = {
  "'": "'",
  "\\": "\\",
  "\r": "r",
  "\n": "n",
  "\u2028": "u2028",
  "\u2029": "u2029"
}, Pt = /\\|'|\r|\n|\u2028|\u2029/g;
function Ct(n) {
  return "\\" + St[n];
}
var Vt = /^\s*(\w|\$)+\s*$/;
function Rt(n, r, t) {
  !r && t && (r = t), r = vr({}, r, v.templateSettings);
  var e = RegExp([
    (r.escape || k).source,
    (r.interpolate || k).source,
    (r.evaluate || k).source
  ].join("|") + "|$", "g"), i = 0, u = "__p+='";
  n.replace(e, function(l, p, m, J, c) {
    return u += n.slice(i, c).replace(Pt, Ct), i = c + l.length, p ? u += `'+
((__t=(` + p + `))==null?'':_.escape(__t))+
'` : m ? u += `'+
((__t=(` + m + `))==null?'':__t)+
'` : J && (u += `';
` + J + `
__p+='`), l;
  }), u += `';
`;
  var f = r.variable;
  if (f) {
    if (!Vt.test(f))
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
  var a;
  try {
    a = new Function(f, "_", u);
  } catch (l) {
    throw l.source = u, l;
  }
  var o = function(l) {
    return a.call(this, l, v);
  };
  return o.source = "function(" + f + `){
` + u + "}", o;
}
function $t(n, r, t) {
  r = U(r);
  var e = r.length;
  if (!e)
    return w(t) ? t.call(n) : t;
  for (var i = 0; i < e; i++) {
    var u = n == null ? void 0 : n[r[i]];
    u === void 0 && (u = t, i = e), n = w(u) ? u.call(n) : u;
  }
  return n;
}
var qt = 0;
function zt(n) {
  var r = ++qt + "";
  return n ? n + r : r;
}
function Lt(n) {
  var r = v(n);
  return r._chain = !0, r;
}
function Ar(n, r, t, e, i) {
  if (!(e instanceof r))
    return n.apply(t, i);
  var u = hr(n.prototype), f = n.apply(u, i);
  return D(f) ? f : u;
}
var V = A(function(n, r) {
  var t = V.placeholder, e = function() {
    for (var i = 0, u = r.length, f = Array(u), a = 0; a < u; a++)
      f[a] = r[a] === t ? arguments[i++] : r[a];
    for (; i < arguments.length; )
      f.push(arguments[i++]);
    return Ar(n, e, this, this, f);
  };
  return e;
});
V.placeholder = v;
const _r = A(function(n, r, t) {
  if (!w(n))
    throw new TypeError("Bind must be called on a function");
  var e = A(function(i) {
    return Ar(n, e, r, this, t.concat(i));
  });
  return e;
}), E = rr(_);
function S(n, r, t, e) {
  if (e = e || [], !r && r !== 0)
    r = 1 / 0;
  else if (r <= 0)
    return e.concat(n);
  for (var i = e.length, u = 0, f = _(n); u < f; u++) {
    var a = n[u];
    if (E(a) && (F(a) || vn(a)))
      if (r > 1)
        S(a, r - 1, t, e), i = e.length;
      else
        for (var o = 0, l = a.length; o < l; )
          e[i++] = a[o++];
    else
      t || (e[i++] = a);
  }
  return e;
}
const Ut = A(function(n, r) {
  r = S(r, !1, !1);
  var t = r.length;
  if (t < 1)
    throw new Error("bindAll must be passed function names");
  for (; t--; ) {
    var e = r[t];
    n[e] = _r(n[e], n);
  }
  return n;
});
function Wt(n, r) {
  var t = function(e) {
    var i = t.cache, u = "" + (r ? r.apply(this, arguments) : e);
    return T(i, u) || (i[u] = n.apply(this, arguments)), i[u];
  };
  return t.cache = {}, t;
}
const Mr = A(function(n, r, t) {
  return setTimeout(function() {
    return n.apply(null, t);
  }, r);
}), Jt = V(Mr, v, 1);
function Xt(n, r, t) {
  var e, i, u, f, a = 0;
  t || (t = {});
  var o = function() {
    a = t.leading === !1 ? 0 : q(), e = null, f = n.apply(i, u), e || (i = u = null);
  }, l = function() {
    var p = q();
    !a && t.leading === !1 && (a = p);
    var m = r - (p - a);
    return i = this, u = arguments, m <= 0 || m > r ? (e && (clearTimeout(e), e = null), a = p, f = n.apply(i, u), e || (i = u = null)) : !e && t.trailing !== !1 && (e = setTimeout(o, m)), f;
  };
  return l.cancel = function() {
    clearTimeout(e), a = 0, e = i = u = null;
  }, l;
}
function Yt(n, r, t) {
  var e, i, u, f, a, o = function() {
    var p = q() - i;
    r > p ? e = setTimeout(o, r - p) : (e = null, t || (f = n.apply(a, u)), e || (u = a = null));
  }, l = A(function(p) {
    return a = this, u = p, i = q(), e || (e = setTimeout(o, r), t && (f = n.apply(a, u))), f;
  });
  return l.cancel = function() {
    clearTimeout(e), e = u = a = null;
  }, l;
}
function Gt(n, r) {
  return V(r, n);
}
function _n(n) {
  return function() {
    return !n.apply(this, arguments);
  };
}
function Ht() {
  var n = arguments, r = n.length - 1;
  return function() {
    for (var t = r, e = n[r].apply(this, arguments); t--; )
      e = n[t].call(this, e);
    return e;
  };
}
function Qt(n, r) {
  return function() {
    if (--n < 1)
      return r.apply(this, arguments);
  };
}
function Er(n, r) {
  var t;
  return function() {
    return --n > 0 && (t = r.apply(this, arguments)), n <= 1 && (r = null), t;
  };
}
const Zt = V(Er, 2);
function Or(n, r, t) {
  r = M(r, t);
  for (var e = y(n), i, u = 0, f = e.length; u < f; u++)
    if (i = e[u], r(n[i], i, n))
      return i;
}
function Ir(n) {
  return function(r, t, e) {
    t = M(t, e);
    for (var i = _(r), u = n > 0 ? 0 : i - 1; u >= 0 && u < i; u += n)
      if (t(r[u], u, r))
        return u;
    return -1;
  };
}
const Mn = Ir(1), Tr = Ir(-1);
function Br(n, r, t, e) {
  t = M(t, e, 1);
  for (var i = t(r), u = 0, f = _(n); u < f; ) {
    var a = Math.floor((u + f) / 2);
    t(n[a]) < i ? u = a + 1 : f = a;
  }
  return u;
}
function Nr(n, r, t) {
  return function(e, i, u) {
    var f = 0, a = _(e);
    if (typeof u == "number")
      n > 0 ? f = u >= 0 ? u : Math.max(u + a, f) : a = u >= 0 ? Math.min(u + 1, a) : u + a + 1;
    else if (t && u && a)
      return u = t(e, i), e[u] === i ? u : -1;
    if (i !== i)
      return u = r(z.call(e, f, a), bn), u >= 0 ? u + f : -1;
    for (u = n > 0 ? f : a - 1; u >= 0 && u < a; u += n)
      if (e[u] === i)
        return u;
    return -1;
  };
}
const Dr = Nr(1, Mn, Br), xt = Nr(-1, Tr);
function un(n, r, t) {
  var e = E(n) ? Mn : Or, i = e(n, r, t);
  if (i !== void 0 && i !== -1)
    return n[i];
}
function Kt(n, r) {
  return un(n, $(r));
}
function I(n, r, t) {
  r = W(r, t);
  var e, i;
  if (E(n))
    for (e = 0, i = n.length; e < i; e++)
      r(n[e], e, n);
  else {
    var u = y(n);
    for (e = 0, i = u.length; e < i; e++)
      r(n[u[e]], u[e], n);
  }
  return n;
}
function N(n, r, t) {
  r = M(r, t);
  for (var e = !E(n) && y(n), i = (e || n).length, u = Array(i), f = 0; f < i; f++) {
    var a = e ? e[f] : f;
    u[f] = r(n[a], a, n);
  }
  return u;
}
function Fr(n) {
  var r = function(t, e, i, u) {
    var f = !E(t) && y(t), a = (f || t).length, o = n > 0 ? 0 : a - 1;
    for (u || (i = t[f ? f[o] : o], o += n); o >= 0 && o < a; o += n) {
      var l = f ? f[o] : o;
      i = e(i, t[l], l, t);
    }
    return i;
  };
  return function(t, e, i, u) {
    var f = arguments.length >= 3;
    return r(t, W(e, u, 4), i, f);
  };
}
const j = Fr(1), zn = Fr(-1);
function P(n, r, t) {
  var e = [];
  return r = M(r, t), I(n, function(i, u, f) {
    r(i, u, f) && e.push(i);
  }), e;
}
function kt(n, r, t) {
  return P(n, _n(M(r)), t);
}
function Ln(n, r, t) {
  r = M(r, t);
  for (var e = !E(n) && y(n), i = (e || n).length, u = 0; u < i; u++) {
    var f = e ? e[u] : u;
    if (!r(n[f], f, n))
      return !1;
  }
  return !0;
}
function Un(n, r, t) {
  r = M(r, t);
  for (var e = !E(n) && y(n), i = (e || n).length, u = 0; u < i; u++) {
    var f = e ? e[u] : u;
    if (r(n[f], f, n))
      return !0;
  }
  return !1;
}
function O(n, r, t, e) {
  return E(n) || (n = C(n)), (typeof t != "number" || e) && (t = 0), Dr(n, r, t) >= 0;
}
const jt = A(function(n, r, t) {
  var e, i;
  return w(r) ? i = r : (r = U(r), e = r.slice(0, -1), r = r[r.length - 1]), N(n, function(u) {
    var f = i;
    if (!f) {
      if (e && e.length && (u = mn(u, e)), u == null)
        return;
      f = u[r];
    }
    return f == null ? f : f.apply(u, t);
  });
});
function En(n, r) {
  return N(n, wn(r));
}
function bt(n, r) {
  return P(n, $(r));
}
function Sr(n, r, t) {
  var e = -1 / 0, i = -1 / 0, u, f;
  if (r == null || typeof r == "number" && typeof n[0] != "object" && n != null) {
    n = E(n) ? n : C(n);
    for (var a = 0, o = n.length; a < o; a++)
      u = n[a], u != null && u > e && (e = u);
  } else
    r = M(r, t), I(n, function(l, p, m) {
      f = r(l, p, m), (f > i || f === -1 / 0 && e === -1 / 0) && (e = l, i = f);
    });
  return e;
}
function ne(n, r, t) {
  var e = 1 / 0, i = 1 / 0, u, f;
  if (r == null || typeof r == "number" && typeof n[0] != "object" && n != null) {
    n = E(n) ? n : C(n);
    for (var a = 0, o = n.length; a < o; a++)
      u = n[a], u != null && u < e && (e = u);
  } else
    r = M(r, t), I(n, function(l, p, m) {
      f = r(l, p, m), (f < i || f === 1 / 0 && e === 1 / 0) && (e = l, i = f);
    });
  return e;
}
var re = /[^\ud800-\udfff]|[\ud800-\udbff][\udc00-\udfff]|[\ud800-\udfff]/g;
function Pr(n) {
  return n ? F(n) ? z.call(n) : cn(n) ? n.match(re) : E(n) ? N(n, dn) : C(n) : [];
}
function Cr(n, r, t) {
  if (r == null || t)
    return E(n) || (n = C(n)), n[en(n.length - 1)];
  var e = Pr(n), i = _(e);
  r = Math.max(Math.min(r, i), 0);
  for (var u = i - 1, f = 0; f < r; f++) {
    var a = en(f, u), o = e[f];
    e[f] = e[a], e[a] = o;
  }
  return e.slice(0, r);
}
function te(n) {
  return Cr(n, 1 / 0);
}
function ee(n, r, t) {
  var e = 0;
  return r = M(r, t), En(N(n, function(i, u, f) {
    return {
      value: i,
      index: e++,
      criteria: r(i, u, f)
    };
  }).sort(function(i, u) {
    var f = i.criteria, a = u.criteria;
    if (f !== a) {
      if (f > a || f === void 0)
        return 1;
      if (f < a || a === void 0)
        return -1;
    }
    return i.index - u.index;
  }), "value");
}
function K(n, r) {
  return function(t, e, i) {
    var u = r ? [[], []] : {};
    return e = M(e, i), I(t, function(f, a) {
      var o = e(f, a, t);
      n(u, f, o);
    }), u;
  };
}
const ue = K(function(n, r, t) {
  T(n, t) ? n[t].push(r) : n[t] = [r];
}), ie = K(function(n, r, t) {
  n[t] = r;
}), fe = K(function(n, r, t) {
  T(n, t) ? n[t]++ : n[t] = 1;
}), ae = K(function(n, r, t) {
  n[t ? 0 : 1].push(r);
}, !0);
function le(n) {
  return n == null ? 0 : E(n) ? n.length : y(n).length;
}
function oe(n, r, t) {
  return r in t;
}
const Vr = A(function(n, r) {
  var t = {}, e = r[0];
  if (n == null)
    return t;
  w(e) ? (r.length > 1 && (e = W(e, r[1])), r = L(n)) : (e = oe, r = S(r, !1, !1), n = Object(n));
  for (var i = 0, u = r.length; i < u; i++) {
    var f = r[i], a = n[f];
    e(a, f, n) && (t[f] = a);
  }
  return t;
}), ce = A(function(n, r) {
  var t = r[0], e;
  return w(t) ? (t = _n(t), r.length > 1 && (e = r[1])) : (r = N(S(r, !1, !1), String), t = function(i, u) {
    return !O(r, u);
  }), Vr(n, t, e);
});
function Rr(n, r, t) {
  return z.call(n, 0, Math.max(0, n.length - (r == null || t ? 1 : r)));
}
function b(n, r, t) {
  return n == null || n.length < 1 ? r == null || t ? void 0 : [] : r == null || t ? n[0] : Rr(n, n.length - r);
}
function G(n, r, t) {
  return z.call(n, r == null || t ? 1 : r);
}
function se(n, r, t) {
  return n == null || n.length < 1 ? r == null || t ? void 0 : [] : r == null || t ? n[n.length - 1] : G(n, Math.max(0, n.length - r));
}
function ve(n) {
  return P(n, Boolean);
}
function he(n, r) {
  return S(n, r, !1);
}
const $r = A(function(n, r) {
  return r = S(r, !0, !0), P(n, function(t) {
    return !O(r, t);
  });
}), pe = A(function(n, r) {
  return $r(n, r);
});
function fn(n, r, t, e) {
  Hn(r) || (e = t, t = r, r = !1), t != null && (t = M(t, e));
  for (var i = [], u = [], f = 0, a = _(n); f < a; f++) {
    var o = n[f], l = t ? t(o, f, n) : o;
    r && !t ? ((!f || u !== l) && i.push(o), u = l) : t ? O(u, l) || (u.push(l), i.push(o)) : O(i, o) || i.push(o);
  }
  return i;
}
const ge = A(function(n) {
  return fn(S(n, !0, !0));
});
function ye(n) {
  for (var r = [], t = arguments.length, e = 0, i = _(n); e < i; e++) {
    var u = n[e];
    if (!O(r, u)) {
      var f;
      for (f = 1; f < t && O(arguments[f], u); f++)
        ;
      f === t && r.push(u);
    }
  }
  return r;
}
function an(n) {
  for (var r = n && Sr(n, _).length || 0, t = Array(r), e = 0; e < r; e++)
    t[e] = En(n, e);
  return t;
}
const me = A(an);
function de(n, r) {
  for (var t = {}, e = 0, i = _(n); e < i; e++)
    r ? t[n[e]] = r[e] : t[n[e][0]] = n[e][1];
  return t;
}
function we(n, r, t) {
  r == null && (r = n || 0, n = 0), t || (t = r < n ? -1 : 1);
  for (var e = Math.max(Math.ceil((r - n) / t), 0), i = Array(e), u = 0; u < e; u++, n += t)
    i[u] = n;
  return i;
}
function Ae(n, r) {
  if (r == null || r < 1)
    return [];
  for (var t = [], e = 0, i = n.length; e < i; )
    t.push(z.call(n, e, e += r));
  return t;
}
function On(n, r) {
  return n._chain ? v(r).chain() : r;
}
function qr(n) {
  return I(tn(n), function(r) {
    var t = v[r] = n[r];
    v.prototype[r] = function() {
      var e = [this._wrapped];
      return Yr.apply(e, arguments), On(this, t.apply(v, e));
    };
  }), v;
}
I(["pop", "push", "reverse", "shift", "sort", "splice", "unshift"], function(n) {
  var r = x[n];
  v.prototype[n] = function() {
    var t = this._wrapped;
    return t != null && (r.apply(t, arguments), (n === "shift" || n === "splice") && t.length === 0 && delete t[0]), On(this, t);
  };
});
I(["concat", "join", "slice"], function(n) {
  var r = x[n];
  v.prototype[n] = function() {
    var t = this._wrapped;
    return t != null && (t = r.apply(t, arguments)), On(this, t);
  };
});
const _e = /* @__PURE__ */ Object.freeze(/* @__PURE__ */ Object.defineProperty({
  __proto__: null,
  VERSION: Jn,
  restArguments: A,
  isObject: D,
  isNull: kr,
  isUndefined: Gn,
  isBoolean: Hn,
  isElement: jr,
  isString: cn,
  isNumber: Qn,
  isDate: br,
  isRegExp: nt,
  isError: rt,
  isSymbol: Zn,
  isArrayBuffer: xn,
  isDataView: H,
  isArray: F,
  isFunction: w,
  isArguments: vn,
  isFinite: it,
  isNaN: bn,
  isTypedArray: er,
  isEmpty: ct,
  isMatch: ir,
  isEqual: st,
  isMap: pt,
  isWeakMap: gt,
  isSet: yt,
  isWeakSet: mt,
  keys: y,
  allKeys: L,
  values: C,
  pairs: dt,
  invert: cr,
  functions: tn,
  methods: tn,
  extend: sr,
  extendOwn: Z,
  assign: Z,
  defaults: vr,
  create: At,
  clone: _t,
  tap: Mt,
  get: gr,
  has: Et,
  mapObject: Ot,
  identity: dn,
  constant: nr,
  noop: mr,
  toPath: pr,
  property: wn,
  propertyOf: It,
  matcher: $,
  matches: $,
  times: Tt,
  random: en,
  now: q,
  escape: Bt,
  unescape: Dt,
  templateSettings: Ft,
  template: Rt,
  result: $t,
  uniqueId: zt,
  chain: Lt,
  iteratee: An,
  partial: V,
  bind: _r,
  bindAll: Ut,
  memoize: Wt,
  delay: Mr,
  defer: Jt,
  throttle: Xt,
  debounce: Yt,
  wrap: Gt,
  negate: _n,
  compose: Ht,
  after: Qt,
  before: Er,
  once: Zt,
  findKey: Or,
  findIndex: Mn,
  findLastIndex: Tr,
  sortedIndex: Br,
  indexOf: Dr,
  lastIndexOf: xt,
  find: un,
  detect: un,
  findWhere: Kt,
  each: I,
  forEach: I,
  map: N,
  collect: N,
  reduce: j,
  foldl: j,
  inject: j,
  reduceRight: zn,
  foldr: zn,
  filter: P,
  select: P,
  reject: kt,
  every: Ln,
  all: Ln,
  some: Un,
  any: Un,
  contains: O,
  includes: O,
  include: O,
  invoke: jt,
  pluck: En,
  where: bt,
  max: Sr,
  min: ne,
  shuffle: te,
  sample: Cr,
  sortBy: ee,
  groupBy: ue,
  indexBy: ie,
  countBy: fe,
  partition: ae,
  toArray: Pr,
  size: le,
  pick: Vr,
  omit: ce,
  first: b,
  head: b,
  take: b,
  initial: Rr,
  last: se,
  rest: G,
  tail: G,
  drop: G,
  compact: ve,
  flatten: he,
  without: pe,
  uniq: fn,
  unique: fn,
  union: ge,
  intersection: ye,
  difference: $r,
  unzip: an,
  transpose: an,
  zip: me,
  object: de,
  range: we,
  chunk: Ae,
  mixin: qr,
  default: v
}, Symbol.toStringTag, { value: "Module" }));
var B = qr(_e);
B._ = B;
(function(n) {
  function r(c) {
    var s = c.charCodeAt(0), h = 1114112, g = 0, In = c.length | 0, X = "";
    switch (s >>> 4) {
      case 12:
      case 13:
        h = (s & 31) << 6 | c.charCodeAt(1) & 63, g = 128 > h ? 0 : 2;
        break;
      case 14:
        h = (s & 15) << 12 | (c.charCodeAt(1) & 63) << 6 | c.charCodeAt(2) & 63, g = 2048 > h ? 0 : 3;
        break;
      case 15:
        s >>> 3 === 30 && (h = (s & 7) << 18 | (c.charCodeAt(1) & 63) << 12 | (c.charCodeAt(2) & 63) << 6 | c.charCodeAt(3), g = 65536 > h ? 0 : 4);
    }
    for (g && (In < g ? g = 0 : 65536 > h ? X = u(h) : 1114112 > h ? (h = h - 65664 | 0, X = u((h >>> 10) + 55296 | 0, (h & 1023) + 56320 | 0)) : g = 0); g < In; g = g + 1 | 0)
      X += "\uFFFD";
    return X;
  }
  function t() {
  }
  function e(c) {
    var s = c.charCodeAt(0) | 0;
    if (55296 <= s && 56319 >= s)
      if (c = c.charCodeAt(1) | 0, 56320 <= c && 57343 >= c) {
        if (s = (s << 10) + c - 56613888 | 0, 65535 < s)
          return u(240 | s >>> 18, 128 | s >>> 12 & 63, 128 | s >>> 6 & 63, 128 | s & 63);
      } else
        s = 65533;
    return 2047 >= s ? u(192 | s >>> 6, 128 | s & 63) : u(224 | s >>> 12, 128 | s >>> 6 & 63, 128 | s & 63);
  }
  function i() {
  }
  var u = String.fromCharCode, f = {}.toString, a = n.SharedArrayBuffer, o = a ? f.call(a) : "", l = n.Uint8Array, p = l || Array, m = f.call((l ? ArrayBuffer : p).prototype);
  a = i.prototype;
  var J = n.TextEncoder;
  t.prototype.decode = function(c) {
    var s = c && c.buffer || c, h = f.call(s);
    if (h !== m && h !== o && c !== void 0)
      throw TypeError("Failed to execute 'decode' on 'TextDecoder': The provided value is not of type '(ArrayBuffer or ArrayBufferView)'");
    c = l ? new p(s) : s, s = "", h = 0;
    for (var g = c.length | 0; h < g; h = h + 32768 | 0)
      s += u.apply(0, c[l ? "subarray" : "slice"](h, h + 32768 | 0));
    return s.replace(/[\xc0-\xff][\x80-\xbf]+|[\x80-\xff]/g, r);
  }, n.TextDecoder || (n.TextDecoder = t), a.encode = function(c) {
    c = c === void 0 ? "" : ("" + c).replace(
      /[\x80-\uD7ff\uDC00-\uFFFF]|[\uD800-\uDBFF][\uDC00-\uDFFF]?/g,
      e
    );
    for (var s = c.length | 0, h = new p(s), g = 0; g < s; g = g + 1 | 0)
      h[g] = c.charCodeAt(g);
    return h;
  }, J || (n.TextEncoder = i);
})("" + void 0 == typeof global ? "" + void 0 == typeof self ? globalThis : self : global);
const Wn = new TextEncoder().encode(JSON.stringify({
  discountApplicationStrategy: ln.First,
  discounts: []
})).buffer;
Xr((n) => {
  let r = JSON.parse(new TextDecoder().decode(n));
  const t = B.get(r, ["discountNode", "metafield", "value"], "{}");
  if (!B.get(r, ["cart", "buyerIdentity", "customer", "metafield", "value"], "{}"))
    return Wn;
  const i = B.chain(r.cart.lines).sortBy((f) => f.quantity).map((f) => Dn(Nn({}, f), { id: B.escape(f.id) })).value();
  return B.reduce(i, (f, a) => f + a.quantity, 0) < 0 ? Wn : new TextEncoder().encode(JSON.stringify({
    discountApplicationStrategy: ln.Maximum,
    discounts: [
      {
        message: "VIP Discount",
        targets: [
          {
            productVariant: {
              id: i[0].id
            }
          }
        ],
        value: {
          percentage: {
            value: t.discountPercentage
          }
        }
      }
    ]
  })).buffer;
});
