var Vr = Object.defineProperty, Fr = Object.defineProperties;
var $r = Object.getOwnPropertyDescriptors;
var An = Object.getOwnPropertySymbols;
var qr = Object.prototype.hasOwnProperty, zr = Object.prototype.propertyIsEnumerable;
var On = (n, r, t) => r in n ? Vr(n, r, { enumerable: !0, configurable: !0, writable: !0, value: t }) : n[r] = t, Mn = (n, r) => {
  for (var t in r || (r = {}))
    qr.call(r, t) && On(n, t, r[t]);
  if (An)
    for (var t of An(r))
      zr.call(r, t) && On(n, t, r[t]);
  return n;
}, In = (n, r) => Fr(n, $r(r));
var rn = /* @__PURE__ */ ((n) => (n.First = "FIRST", n.Maximum = "MAXIMUM", n))(rn || {}), zn = "1.13.6", En = typeof self == "object" && self.self === self && self || typeof global == "object" && global.global === global && global || Function("return this")() || {}, X = Array.prototype, tn = Object.prototype, Nn = typeof Symbol != "undefined" ? Symbol.prototype : null, Lr = X.push, F = X.slice, D = tn.toString, Ur = tn.hasOwnProperty, Ln = typeof ArrayBuffer != "undefined", Jr = typeof DataView != "undefined", Cr = Array.isArray, Sn = Object.keys, Bn = Object.create, Pn = Ln && ArrayBuffer.isView, Wr = isNaN, Xr = isFinite, Un = !{ toString: null }.propertyIsEnumerable("toString"), Tn = [
  "valueOf",
  "isPrototypeOf",
  "toString",
  "propertyIsEnumerable",
  "hasOwnProperty",
  "toLocaleString"
], Yr = Math.pow(2, 53) - 1;
function y(n, r) {
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
function E(n) {
  var r = typeof n;
  return r === "function" || r === "object" && !!n;
}
function Gr(n) {
  return n === null;
}
function Jn(n) {
  return n === void 0;
}
function Cn(n) {
  return n === !0 || n === !1 || D.call(n) === "[object Boolean]";
}
function Hr(n) {
  return !!(n && n.nodeType === 1);
}
function p(n) {
  var r = "[object " + n + "]";
  return function(t) {
    return D.call(t) === r;
  };
}
const en = p("String"), Wn = p("Number"), Qr = p("Date"), Zr = p("RegExp"), Kr = p("Error"), Xn = p("Symbol"), Yn = p("ArrayBuffer");
var Gn = p("Function"), xr = En.document && En.document.childNodes;
typeof /./ != "function" && typeof Int8Array != "object" && typeof xr != "function" && (Gn = function(n) {
  return typeof n == "function" || !1;
});
const g = Gn, Hn = p("Object");
var Qn = Jr && Hn(new DataView(new ArrayBuffer(8))), un = typeof Map != "undefined" && Hn(/* @__PURE__ */ new Map()), kr = p("DataView");
function br(n) {
  return n != null && g(n.getInt8) && Yn(n.buffer);
}
const U = Qn ? br : kr, N = Cr || p("Array");
function O(n, r) {
  return n != null && Ur.call(n, r);
}
var Z = p("Arguments");
(function() {
  Z(arguments) || (Z = function(n) {
    return O(n, "callee");
  });
})();
const fn = Z;
function jr(n) {
  return !Xn(n) && Xr(n) && !isNaN(parseFloat(n));
}
function Zn(n) {
  return Wn(n) && Wr(n);
}
function Kn(n) {
  return function() {
    return n;
  };
}
function xn(n) {
  return function(r) {
    var t = n(r);
    return typeof t == "number" && t >= 0 && t <= Yr;
  };
}
function kn(n) {
  return function(r) {
    return r == null ? void 0 : r[n];
  };
}
const J = kn("byteLength"), nt = xn(J);
var rt = /\[object ((I|Ui)nt(8|16|32)|Float(32|64)|Uint8Clamped|Big(I|Ui)nt64)Array\]/;
function tt(n) {
  return Pn ? Pn(n) && !U(n) : nt(n) && rt.test(D.call(n));
}
const bn = Ln ? tt : Kn(!1), m = kn("length");
function et(n) {
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
function jn(n, r) {
  r = et(r);
  var t = Tn.length, e = n.constructor, i = g(e) && e.prototype || tn, u = "constructor";
  for (O(n, u) && !r.contains(u) && r.push(u); t--; )
    u = Tn[t], u in n && n[u] !== i[u] && !r.contains(u) && r.push(u);
}
function v(n) {
  if (!E(n))
    return [];
  if (Sn)
    return Sn(n);
  var r = [];
  for (var t in n)
    O(n, t) && r.push(t);
  return Un && jn(n, r), r;
}
function ut(n) {
  if (n == null)
    return !0;
  var r = m(n);
  return typeof r == "number" && (N(n) || en(n) || fn(n)) ? r === 0 : m(v(n)) === 0;
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
o.prototype.value = function() {
  return this._wrapped;
};
o.prototype.valueOf = o.prototype.toJSON = o.prototype.value;
o.prototype.toString = function() {
  return String(this._wrapped);
};
function Dn(n) {
  return new Uint8Array(
    n.buffer || n,
    n.byteOffset || 0,
    J(n)
  );
}
var Rn = "[object DataView]";
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
  if (Qn && i == "[object Object]" && U(n)) {
    if (!U(r))
      return !1;
    i = Rn;
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
    case Rn:
      return rr(Dn(n), Dn(r), t, e);
  }
  var u = i === "[object Array]";
  if (!u && bn(n)) {
    var f = J(n);
    if (f !== J(r))
      return !1;
    if (n.buffer === r.buffer && n.byteOffset === r.byteOffset)
      return !0;
    u = !0;
  }
  if (!u) {
    if (typeof n != "object" || typeof r != "object")
      return !1;
    var a = n.constructor, c = r.constructor;
    if (a !== c && !(g(a) && a instanceof a && g(c) && c instanceof c) && "constructor" in n && "constructor" in r)
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
      if (!K(n[l], r[l], t, e))
        return !1;
  } else {
    var s = v(n), h;
    if (l = s.length, v(r).length !== l)
      return !1;
    for (; l--; )
      if (h = s[l], !(O(r, h) && K(n[h], r[h], t, e)))
        return !1;
  }
  return t.pop(), e.pop(), !0;
}
function it(n, r) {
  return K(n, r);
}
function $(n) {
  if (!E(n))
    return [];
  var r = [];
  for (var t in n)
    r.push(t);
  return Un && jn(n, r), r;
}
function an(n) {
  var r = m(n);
  return function(t) {
    if (t == null)
      return !1;
    var e = $(t);
    if (m(e))
      return !1;
    for (var i = 0; i < r; i++)
      if (!g(t[n[i]]))
        return !1;
    return n !== ur || !g(t[ln]);
  };
}
var ln = "forEach", tr = "has", cn = ["clear", "delete"], er = ["get", tr, "set"], ft = cn.concat(ln, er), ur = cn.concat(er), at = ["add"].concat(cn, ln, tr);
const lt = un ? an(ft) : p("Map"), ct = un ? an(ur) : p("WeakMap"), ot = un ? an(at) : p("Set"), st = p("WeakSet");
function P(n) {
  for (var r = v(n), t = r.length, e = Array(t), i = 0; i < t; i++)
    e[i] = n[r[i]];
  return e;
}
function vt(n) {
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
  return function(t) {
    var e = arguments.length;
    if (r && (t = Object(t)), e < 2 || t == null)
      return t;
    for (var i = 1; i < e; i++)
      for (var u = arguments[i], f = n(u), a = f.length, c = 0; c < a; c++) {
        var l = f[c];
        (!r || t[l] === void 0) && (t[l] = u[l]);
      }
    return t;
  };
}
const fr = on($), C = on(v), ar = on($, !0);
function pt() {
  return function() {
  };
}
function lr(n) {
  if (!E(n))
    return {};
  if (Bn)
    return Bn(n);
  var r = pt();
  r.prototype = n;
  var t = new r();
  return r.prototype = null, t;
}
function ht(n, r) {
  var t = lr(n);
  return r && C(t, r), t;
}
function gt(n) {
  return E(n) ? N(n) ? n.slice() : fr({}, n) : n;
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
  return Jn(e) ? t : e;
}
function mt(n, r) {
  r = q(r);
  for (var t = r.length, e = 0; e < t; e++) {
    var i = r[e];
    if (!O(n, i))
      return !1;
    n = n[i];
  }
  return !!t;
}
function vn(n) {
  return n;
}
function R(n) {
  return n = C({}, n), function(r) {
    return nr(r, n);
  };
}
function pn(n) {
  return n = q(n), function(r) {
    return sn(r, n);
  };
}
function z(n, r, t) {
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
function sr(n, r, t) {
  return n == null ? vn : g(n) ? z(n, r, t) : E(n) && !N(n) ? R(n) : pn(n);
}
function hn(n, r) {
  return sr(n, r, 1 / 0);
}
o.iteratee = hn;
function d(n, r, t) {
  return o.iteratee !== hn ? o.iteratee(n, r) : sr(n, r, t);
}
function dt(n, r, t) {
  r = d(r, t);
  for (var e = v(n), i = e.length, u = {}, f = 0; f < i; f++) {
    var a = e[f];
    u[a] = r(n[a], a, n);
  }
  return u;
}
function vr() {
}
function wt(n) {
  return n == null ? vr : function(r) {
    return or(n, r);
  };
}
function _t(n, r, t) {
  var e = Array(Math.max(0, n));
  r = z(r, t, 1);
  for (var i = 0; i < n; i++)
    e[i] = r(i);
  return e;
}
function k(n, r) {
  return r == null && (r = n, n = 0), n + Math.floor(Math.random() * (r - n + 1));
}
const V = Date.now || function() {
  return new Date().getTime();
};
function pr(n) {
  var r = function(u) {
    return n[u];
  }, t = "(?:" + v(n).join("|") + ")", e = RegExp(t), i = RegExp(t, "g");
  return function(u) {
    return u = u == null ? "" : "" + u, e.test(u) ? u.replace(i, r) : u;
  };
}
const hr = {
  "&": "&amp;",
  "<": "&lt;",
  ">": "&gt;",
  '"': "&quot;",
  "'": "&#x27;",
  "`": "&#x60;"
}, At = pr(hr), Ot = ir(hr), Mt = pr(Ot), It = o.templateSettings = {
  evaluate: /<%([\s\S]+?)%>/g,
  interpolate: /<%=([\s\S]+?)%>/g,
  escape: /<%-([\s\S]+?)%>/g
};
var G = /(.)^/, Et = {
  "'": "'",
  "\\": "\\",
  "\r": "r",
  "\n": "n",
  "\u2028": "u2028",
  "\u2029": "u2029"
}, Nt = /\\|'|\r|\n|\u2028|\u2029/g;
function St(n) {
  return "\\" + Et[n];
}
var Bt = /^\s*(\w|\$)+\s*$/;
function Pt(n, r, t) {
  !r && t && (r = t), r = ar({}, r, o.templateSettings);
  var e = RegExp([
    (r.escape || G).source,
    (r.interpolate || G).source,
    (r.evaluate || G).source
  ].join("|") + "|$", "g"), i = 0, u = "__p+='";
  n.replace(e, function(l, s, h, wn, _n) {
    return u += n.slice(i, _n).replace(Nt, St), i = _n + l.length, s ? u += `'+
((__t=(` + s + `))==null?'':_.escape(__t))+
'` : h ? u += `'+
((__t=(` + h + `))==null?'':__t)+
'` : wn && (u += `';
` + wn + `
__p+='`), l;
  }), u += `';
`;
  var f = r.variable;
  if (f) {
    if (!Bt.test(f))
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
  var c = function(l) {
    return a.call(this, l, o);
  };
  return c.source = "function(" + f + `){
` + u + "}", c;
}
function Tt(n, r, t) {
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
var Dt = 0;
function Rt(n) {
  var r = ++Dt + "";
  return n ? n + r : r;
}
function Vt(n) {
  var r = o(n);
  return r._chain = !0, r;
}
function gr(n, r, t, e, i) {
  if (!(e instanceof r))
    return n.apply(t, i);
  var u = lr(n.prototype), f = n.apply(u, i);
  return E(f) ? f : u;
}
var T = y(function(n, r) {
  var t = T.placeholder, e = function() {
    for (var i = 0, u = r.length, f = Array(u), a = 0; a < u; a++)
      f[a] = r[a] === t ? arguments[i++] : r[a];
    for (; i < arguments.length; )
      f.push(arguments[i++]);
    return gr(n, e, this, this, f);
  };
  return e;
});
T.placeholder = o;
const yr = y(function(n, r, t) {
  if (!g(n))
    throw new TypeError("Bind must be called on a function");
  var e = y(function(i) {
    return gr(n, e, r, this, t.concat(i));
  });
  return e;
}), w = xn(m);
function S(n, r, t, e) {
  if (e = e || [], !r && r !== 0)
    r = 1 / 0;
  else if (r <= 0)
    return e.concat(n);
  for (var i = e.length, u = 0, f = m(n); u < f; u++) {
    var a = n[u];
    if (w(a) && (N(a) || fn(a)))
      if (r > 1)
        S(a, r - 1, t, e), i = e.length;
      else
        for (var c = 0, l = a.length; c < l; )
          e[i++] = a[c++];
    else
      t || (e[i++] = a);
  }
  return e;
}
const Ft = y(function(n, r) {
  r = S(r, !1, !1);
  var t = r.length;
  if (t < 1)
    throw new Error("bindAll must be passed function names");
  for (; t--; ) {
    var e = r[t];
    n[e] = yr(n[e], n);
  }
  return n;
});
function $t(n, r) {
  var t = function(e) {
    var i = t.cache, u = "" + (r ? r.apply(this, arguments) : e);
    return O(i, u) || (i[u] = n.apply(this, arguments)), i[u];
  };
  return t.cache = {}, t;
}
const mr = y(function(n, r, t) {
  return setTimeout(function() {
    return n.apply(null, t);
  }, r);
}), qt = T(mr, o, 1);
function zt(n, r, t) {
  var e, i, u, f, a = 0;
  t || (t = {});
  var c = function() {
    a = t.leading === !1 ? 0 : V(), e = null, f = n.apply(i, u), e || (i = u = null);
  }, l = function() {
    var s = V();
    !a && t.leading === !1 && (a = s);
    var h = r - (s - a);
    return i = this, u = arguments, h <= 0 || h > r ? (e && (clearTimeout(e), e = null), a = s, f = n.apply(i, u), e || (i = u = null)) : !e && t.trailing !== !1 && (e = setTimeout(c, h)), f;
  };
  return l.cancel = function() {
    clearTimeout(e), a = 0, e = i = u = null;
  }, l;
}
function Lt(n, r, t) {
  var e, i, u, f, a, c = function() {
    var s = V() - i;
    r > s ? e = setTimeout(c, r - s) : (e = null, t || (f = n.apply(a, u)), e || (u = a = null));
  }, l = y(function(s) {
    return a = this, u = s, i = V(), e || (e = setTimeout(c, r), t && (f = n.apply(a, u))), f;
  });
  return l.cancel = function() {
    clearTimeout(e), e = u = a = null;
  }, l;
}
function Ut(n, r) {
  return T(r, n);
}
function gn(n) {
  return function() {
    return !n.apply(this, arguments);
  };
}
function Jt() {
  var n = arguments, r = n.length - 1;
  return function() {
    for (var t = r, e = n[r].apply(this, arguments); t--; )
      e = n[t].call(this, e);
    return e;
  };
}
function Ct(n, r) {
  return function() {
    if (--n < 1)
      return r.apply(this, arguments);
  };
}
function dr(n, r) {
  var t;
  return function() {
    return --n > 0 && (t = r.apply(this, arguments)), n <= 1 && (r = null), t;
  };
}
const Wt = T(dr, 2);
function wr(n, r, t) {
  r = d(r, t);
  for (var e = v(n), i, u = 0, f = e.length; u < f; u++)
    if (i = e[u], r(n[i], i, n))
      return i;
}
function _r(n) {
  return function(r, t, e) {
    t = d(t, e);
    for (var i = m(r), u = n > 0 ? 0 : i - 1; u >= 0 && u < i; u += n)
      if (t(r[u], u, r))
        return u;
    return -1;
  };
}
const yn = _r(1), Ar = _r(-1);
function Or(n, r, t, e) {
  t = d(t, e, 1);
  for (var i = t(r), u = 0, f = m(n); u < f; ) {
    var a = Math.floor((u + f) / 2);
    t(n[a]) < i ? u = a + 1 : f = a;
  }
  return u;
}
function Mr(n, r, t) {
  return function(e, i, u) {
    var f = 0, a = m(e);
    if (typeof u == "number")
      n > 0 ? f = u >= 0 ? u : Math.max(u + a, f) : a = u >= 0 ? Math.min(u + 1, a) : u + a + 1;
    else if (t && u && a)
      return u = t(e, i), e[u] === i ? u : -1;
    if (i !== i)
      return u = r(F.call(e, f, a), Zn), u >= 0 ? u + f : -1;
    for (u = n > 0 ? f : a - 1; u >= 0 && u < a; u += n)
      if (e[u] === i)
        return u;
    return -1;
  };
}
const Ir = Mr(1, yn, Or), Xt = Mr(-1, Ar);
function b(n, r, t) {
  var e = w(n) ? yn : wr, i = e(n, r, t);
  if (i !== void 0 && i !== -1)
    return n[i];
}
function Yt(n, r) {
  return b(n, R(r));
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
function I(n, r, t) {
  r = d(r, t);
  for (var e = !w(n) && v(n), i = (e || n).length, u = Array(i), f = 0; f < i; f++) {
    var a = e ? e[f] : f;
    u[f] = r(n[a], a, n);
  }
  return u;
}
function Er(n) {
  var r = function(t, e, i, u) {
    var f = !w(t) && v(t), a = (f || t).length, c = n > 0 ? 0 : a - 1;
    for (u || (i = t[f ? f[c] : c], c += n); c >= 0 && c < a; c += n) {
      var l = f ? f[c] : c;
      i = e(i, t[l], l, t);
    }
    return i;
  };
  return function(t, e, i, u) {
    var f = arguments.length >= 3;
    return r(t, z(e, u, 4), i, f);
  };
}
const H = Er(1), Vn = Er(-1);
function B(n, r, t) {
  var e = [];
  return r = d(r, t), A(n, function(i, u, f) {
    r(i, u, f) && e.push(i);
  }), e;
}
function Gt(n, r, t) {
  return B(n, gn(d(r)), t);
}
function Fn(n, r, t) {
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
  return w(n) || (n = P(n)), (typeof t != "number" || e) && (t = 0), Ir(n, r, t) >= 0;
}
const Ht = y(function(n, r, t) {
  var e, i;
  return g(r) ? i = r : (r = q(r), e = r.slice(0, -1), r = r[r.length - 1]), I(n, function(u) {
    var f = i;
    if (!f) {
      if (e && e.length && (u = sn(u, e)), u == null)
        return;
      f = u[r];
    }
    return f == null ? f : f.apply(u, t);
  });
});
function mn(n, r) {
  return I(n, pn(r));
}
function Qt(n, r) {
  return B(n, R(r));
}
function Nr(n, r, t) {
  var e = -1 / 0, i = -1 / 0, u, f;
  if (r == null || typeof r == "number" && typeof n[0] != "object" && n != null) {
    n = w(n) ? n : P(n);
    for (var a = 0, c = n.length; a < c; a++)
      u = n[a], u != null && u > e && (e = u);
  } else
    r = d(r, t), A(n, function(l, s, h) {
      f = r(l, s, h), (f > i || f === -1 / 0 && e === -1 / 0) && (e = l, i = f);
    });
  return e;
}
function Zt(n, r, t) {
  var e = 1 / 0, i = 1 / 0, u, f;
  if (r == null || typeof r == "number" && typeof n[0] != "object" && n != null) {
    n = w(n) ? n : P(n);
    for (var a = 0, c = n.length; a < c; a++)
      u = n[a], u != null && u < e && (e = u);
  } else
    r = d(r, t), A(n, function(l, s, h) {
      f = r(l, s, h), (f < i || f === 1 / 0 && e === 1 / 0) && (e = l, i = f);
    });
  return e;
}
var Kt = /[^\ud800-\udfff]|[\ud800-\udbff][\udc00-\udfff]|[\ud800-\udfff]/g;
function Sr(n) {
  return n ? N(n) ? F.call(n) : en(n) ? n.match(Kt) : w(n) ? I(n, vn) : P(n) : [];
}
function Br(n, r, t) {
  if (r == null || t)
    return w(n) || (n = P(n)), n[k(n.length - 1)];
  var e = Sr(n), i = m(e);
  r = Math.max(Math.min(r, i), 0);
  for (var u = i - 1, f = 0; f < r; f++) {
    var a = k(f, u), c = e[f];
    e[f] = e[a], e[a] = c;
  }
  return e.slice(0, r);
}
function xt(n) {
  return Br(n, 1 / 0);
}
function kt(n, r, t) {
  var e = 0;
  return r = d(r, t), mn(I(n, function(i, u, f) {
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
function Y(n, r) {
  return function(t, e, i) {
    var u = r ? [[], []] : {};
    return e = d(e, i), A(t, function(f, a) {
      var c = e(f, a, t);
      n(u, f, c);
    }), u;
  };
}
const bt = Y(function(n, r, t) {
  O(n, t) ? n[t].push(r) : n[t] = [r];
}), jt = Y(function(n, r, t) {
  n[t] = r;
}), ne = Y(function(n, r, t) {
  O(n, t) ? n[t]++ : n[t] = 1;
}), re = Y(function(n, r, t) {
  n[t ? 0 : 1].push(r);
}, !0);
function te(n) {
  return n == null ? 0 : w(n) ? n.length : v(n).length;
}
function ee(n, r, t) {
  return r in t;
}
const Pr = y(function(n, r) {
  var t = {}, e = r[0];
  if (n == null)
    return t;
  g(e) ? (r.length > 1 && (e = z(e, r[1])), r = $(n)) : (e = ee, r = S(r, !1, !1), n = Object(n));
  for (var i = 0, u = r.length; i < u; i++) {
    var f = r[i], a = n[f];
    e(a, f, n) && (t[f] = a);
  }
  return t;
}), ue = y(function(n, r) {
  var t = r[0], e;
  return g(t) ? (t = gn(t), r.length > 1 && (e = r[1])) : (r = I(S(r, !1, !1), String), t = function(i, u) {
    return !_(r, u);
  }), Pr(n, t, e);
});
function Tr(n, r, t) {
  return F.call(n, 0, Math.max(0, n.length - (r == null || t ? 1 : r)));
}
function Q(n, r, t) {
  return n == null || n.length < 1 ? r == null || t ? void 0 : [] : r == null || t ? n[0] : Tr(n, n.length - r);
}
function L(n, r, t) {
  return F.call(n, r == null || t ? 1 : r);
}
function ie(n, r, t) {
  return n == null || n.length < 1 ? r == null || t ? void 0 : [] : r == null || t ? n[n.length - 1] : L(n, Math.max(0, n.length - r));
}
function fe(n) {
  return B(n, Boolean);
}
function ae(n, r) {
  return S(n, r, !1);
}
const Dr = y(function(n, r) {
  return r = S(r, !0, !0), B(n, function(t) {
    return !_(r, t);
  });
}), le = y(function(n, r) {
  return Dr(n, r);
});
function j(n, r, t, e) {
  Cn(r) || (e = t, t = r, r = !1), t != null && (t = d(t, e));
  for (var i = [], u = [], f = 0, a = m(n); f < a; f++) {
    var c = n[f], l = t ? t(c, f, n) : c;
    r && !t ? ((!f || u !== l) && i.push(c), u = l) : t ? _(u, l) || (u.push(l), i.push(c)) : _(i, c) || i.push(c);
  }
  return i;
}
const ce = y(function(n) {
  return j(S(n, !0, !0));
});
function oe(n) {
  for (var r = [], t = arguments.length, e = 0, i = m(n); e < i; e++) {
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
  for (var r = n && Nr(n, m).length || 0, t = Array(r), e = 0; e < r; e++)
    t[e] = mn(n, e);
  return t;
}
const se = y(nn);
function ve(n, r) {
  for (var t = {}, e = 0, i = m(n); e < i; e++)
    r ? t[n[e]] = r[e] : t[n[e][0]] = n[e][1];
  return t;
}
function pe(n, r, t) {
  r == null && (r = n || 0, n = 0), t || (t = r < n ? -1 : 1);
  for (var e = Math.max(Math.ceil((r - n) / t), 0), i = Array(e), u = 0; u < e; u++, n += t)
    i[u] = n;
  return i;
}
function he(n, r) {
  if (r == null || r < 1)
    return [];
  for (var t = [], e = 0, i = n.length; e < i; )
    t.push(F.call(n, e, e += r));
  return t;
}
function dn(n, r) {
  return n._chain ? o(r).chain() : r;
}
function Rr(n) {
  return A(x(n), function(r) {
    var t = o[r] = n[r];
    o.prototype[r] = function() {
      var e = [this._wrapped];
      return Lr.apply(e, arguments), dn(this, t.apply(o, e));
    };
  }), o;
}
A(["pop", "push", "reverse", "shift", "sort", "splice", "unshift"], function(n) {
  var r = X[n];
  o.prototype[n] = function() {
    var t = this._wrapped;
    return t != null && (r.apply(t, arguments), (n === "shift" || n === "splice") && t.length === 0 && delete t[0]), dn(this, t);
  };
});
A(["concat", "join", "slice"], function(n) {
  var r = X[n];
  o.prototype[n] = function() {
    var t = this._wrapped;
    return t != null && (t = r.apply(t, arguments)), dn(this, t);
  };
});
const ge = /* @__PURE__ */ Object.freeze(/* @__PURE__ */ Object.defineProperty({
  __proto__: null,
  VERSION: zn,
  restArguments: y,
  isObject: E,
  isNull: Gr,
  isUndefined: Jn,
  isBoolean: Cn,
  isElement: Hr,
  isString: en,
  isNumber: Wn,
  isDate: Qr,
  isRegExp: Zr,
  isError: Kr,
  isSymbol: Xn,
  isArrayBuffer: Yn,
  isDataView: U,
  isArray: N,
  isFunction: g,
  isArguments: fn,
  isFinite: jr,
  isNaN: Zn,
  isTypedArray: bn,
  isEmpty: ut,
  isMatch: nr,
  isEqual: it,
  isMap: lt,
  isWeakMap: ct,
  isSet: ot,
  isWeakSet: st,
  keys: v,
  allKeys: $,
  values: P,
  pairs: vt,
  invert: ir,
  functions: x,
  methods: x,
  extend: fr,
  extendOwn: C,
  assign: C,
  defaults: ar,
  create: ht,
  clone: gt,
  tap: yt,
  get: or,
  has: mt,
  mapObject: dt,
  identity: vn,
  constant: Kn,
  noop: vr,
  toPath: cr,
  property: pn,
  propertyOf: wt,
  matcher: R,
  matches: R,
  times: _t,
  random: k,
  now: V,
  escape: At,
  unescape: Mt,
  templateSettings: It,
  template: Pt,
  result: Tt,
  uniqueId: Rt,
  chain: Vt,
  iteratee: hn,
  partial: T,
  bind: yr,
  bindAll: Ft,
  memoize: $t,
  delay: mr,
  defer: qt,
  throttle: zt,
  debounce: Lt,
  wrap: Ut,
  negate: gn,
  compose: Jt,
  after: Ct,
  before: dr,
  once: Wt,
  findKey: wr,
  findIndex: yn,
  findLastIndex: Ar,
  sortedIndex: Or,
  indexOf: Ir,
  lastIndexOf: Xt,
  find: b,
  detect: b,
  findWhere: Yt,
  each: A,
  forEach: A,
  map: I,
  collect: I,
  reduce: H,
  foldl: H,
  inject: H,
  reduceRight: Vn,
  foldr: Vn,
  filter: B,
  select: B,
  reject: Gt,
  every: Fn,
  all: Fn,
  some: $n,
  any: $n,
  contains: _,
  includes: _,
  include: _,
  invoke: Ht,
  pluck: mn,
  where: Qt,
  max: Nr,
  min: Zt,
  shuffle: xt,
  sample: Br,
  sortBy: kt,
  groupBy: bt,
  indexBy: jt,
  countBy: ne,
  partition: re,
  toArray: Sr,
  size: te,
  pick: Pr,
  omit: ue,
  first: Q,
  head: Q,
  take: Q,
  initial: Tr,
  last: ie,
  rest: L,
  tail: L,
  drop: L,
  compact: fe,
  flatten: ae,
  without: le,
  uniq: j,
  unique: j,
  union: ce,
  intersection: oe,
  difference: Dr,
  unzip: nn,
  transpose: nn,
  zip: se,
  object: ve,
  range: pe,
  chunk: he,
  mixin: Rr,
  default: o
}, Symbol.toStringTag, { value: "Module" }));
var M = Rr(ge);
M._ = M;
const qn = {
  discountApplicationStrategy: rn.First,
  discounts: []
};
function ye(n) {
  const r = JSON.parse(
    M.get(n, ["discountNode", "metafield", "value"], "{}")
  );
  if (!JSON.parse(
    M.get(n, ["cart", "buyerIdentity", "customer", "metafield", "value"], "{}")
  ))
    return qn;
  const e = M.chain(n.cart.lines).sortBy((u) => u.quantity).map((u) => In(Mn({}, u), { id: M.escape(u.id) })).value();
  return M.reduce(e, (u, f) => u + f.quantity, 0) < 0 ? qn : {
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
}
let W = new Uint8Array(1024);
const me = Javy.IO.readSync(0, W);
W = W.subarray(0, me);
const de = new TextEncoder().encode(JSON.stringify(ye(JSON.parse(new TextDecoder().decode(W)))));
Javy.IO.writeSync(1, de);
