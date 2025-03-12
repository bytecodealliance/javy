var Rr = Object.defineProperty, Vr = Object.defineProperties;
var Fr = Object.getOwnPropertyDescriptors;
var _n = Object.getOwnPropertySymbols;
var $r = Object.prototype.hasOwnProperty, qr = Object.prototype.propertyIsEnumerable;
var An = (n, r, t) => r in n ? Rr(n, r, { enumerable: !0, configurable: !0, writable: !0, value: t }) : n[r] = t, On = (n, r) => {
  for (var t in r || (r = {}))
    $r.call(r, t) && An(n, t, r[t]);
  if (_n)
    for (var t of _n(r))
      qr.call(r, t) && An(n, t, r[t]);
  return n;
}, Mn = (n, r) => Vr(n, Fr(r));
var qn = "1.13.6", En = typeof self == "object" && self.self === self && self || typeof global == "object" && global.global === global && global || Function("return this")() || {}, X = Array.prototype, rn = Object.prototype, In = typeof Symbol != "undefined" ? Symbol.prototype : null, zr = X.push, F = X.slice, D = rn.toString, Lr = rn.hasOwnProperty, zn = typeof ArrayBuffer != "undefined", Jr = typeof DataView != "undefined", Ur = Array.isArray, Nn = Object.keys, Sn = Object.create, Bn = zn && ArrayBuffer.isView, Cr = isNaN, Wr = isFinite, Ln = !{ toString: null }.propertyIsEnumerable("toString"), Pn = [
  "valueOf",
  "isPrototypeOf",
  "toString",
  "propertyIsEnumerable",
  "hasOwnProperty",
  "toLocaleString"
], Xr = Math.pow(2, 53) - 1;
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
function I(n) {
  var r = typeof n;
  return r === "function" || r === "object" && !!n;
}
function Yr(n) {
  return n === null;
}
function Jn(n) {
  return n === void 0;
}
function Un(n) {
  return n === !0 || n === !1 || D.call(n) === "[object Boolean]";
}
function Gr(n) {
  return !!(n && n.nodeType === 1);
}
function p(n) {
  var r = "[object " + n + "]";
  return function(t) {
    return D.call(t) === r;
  };
}
const tn = p("String"), Cn = p("Number"), Hr = p("Date"), Qr = p("RegExp"), Zr = p("Error"), Wn = p("Symbol"), Xn = p("ArrayBuffer");
var Yn = p("Function"), Kr = En.document && En.document.childNodes;
typeof /./ != "function" && typeof Int8Array != "object" && typeof Kr != "function" && (Yn = function(n) {
  return typeof n == "function" || !1;
});
const g = Yn, Gn = p("Object");
var Hn = Jr && Gn(new DataView(new ArrayBuffer(8))), en = typeof Map != "undefined" && Gn(/* @__PURE__ */ new Map()), xr = p("DataView");
function kr(n) {
  return n != null && g(n.getInt8) && Xn(n.buffer);
}
const J = Hn ? kr : xr, N = Ur || p("Array");
function O(n, r) {
  return n != null && Lr.call(n, r);
}
var Z = p("Arguments");
(function() {
  Z(arguments) || (Z = function(n) {
    return O(n, "callee");
  });
})();
const un = Z;
function br(n) {
  return !Wn(n) && Wr(n) && !isNaN(parseFloat(n));
}
function Qn(n) {
  return Cn(n) && Cr(n);
}
function Zn(n) {
  return function() {
    return n;
  };
}
function Kn(n) {
  return function(r) {
    var t = n(r);
    return typeof t == "number" && t >= 0 && t <= Xr;
  };
}
function xn(n) {
  return function(r) {
    return r == null ? void 0 : r[n];
  };
}
const U = xn("byteLength"), jr = Kn(U);
var nt = /\[object ((I|Ui)nt(8|16|32)|Float(32|64)|Uint8Clamped|Big(I|Ui)nt64)Array\]/;
function rt(n) {
  return Bn ? Bn(n) && !J(n) : jr(n) && nt.test(D.call(n));
}
const kn = zn ? rt : Zn(!1), m = xn("length");
function tt(n) {
  for (var r = {}, t = n.length, e = 0; e < t; ++e) r[n[e]] = !0;
  return {
    contains: function(i) {
      return r[i] === !0;
    },
    push: function(i) {
      return r[i] = !0, n.push(i);
    }
  };
}
function bn(n, r) {
  r = tt(r);
  var t = Pn.length, e = n.constructor, i = g(e) && e.prototype || rn, u = "constructor";
  for (O(n, u) && !r.contains(u) && r.push(u); t--; )
    u = Pn[t], u in n && n[u] !== i[u] && !r.contains(u) && r.push(u);
}
function v(n) {
  if (!I(n)) return [];
  if (Nn) return Nn(n);
  var r = [];
  for (var t in n) O(n, t) && r.push(t);
  return Ln && bn(n, r), r;
}
function et(n) {
  if (n == null) return !0;
  var r = m(n);
  return typeof r == "number" && (N(n) || tn(n) || un(n)) ? r === 0 : m(v(n)) === 0;
}
function jn(n, r) {
  var t = v(r), e = t.length;
  if (n == null) return !e;
  for (var i = Object(n), u = 0; u < e; u++) {
    var f = t[u];
    if (r[f] !== i[f] || !(f in i)) return !1;
  }
  return !0;
}
function o(n) {
  if (n instanceof o) return n;
  if (!(this instanceof o)) return new o(n);
  this._wrapped = n;
}
o.VERSION = qn;
o.prototype.value = function() {
  return this._wrapped;
};
o.prototype.valueOf = o.prototype.toJSON = o.prototype.value;
o.prototype.toString = function() {
  return String(this._wrapped);
};
function Tn(n) {
  return new Uint8Array(
    n.buffer || n,
    n.byteOffset || 0,
    U(n)
  );
}
var Dn = "[object DataView]";
function K(n, r, t, e) {
  if (n === r) return n !== 0 || 1 / n === 1 / r;
  if (n == null || r == null) return !1;
  if (n !== n) return r !== r;
  var i = typeof n;
  return i !== "function" && i !== "object" && typeof r != "object" ? !1 : nr(n, r, t, e);
}
function nr(n, r, t, e) {
  n instanceof o && (n = n._wrapped), r instanceof o && (r = r._wrapped);
  var i = D.call(n);
  if (i !== D.call(r)) return !1;
  if (Hn && i == "[object Object]" && J(n)) {
    if (!J(r)) return !1;
    i = Dn;
  }
  switch (i) {
    // These types are compared by value.
    case "[object RegExp]":
    // RegExps are coerced to strings for comparison (Note: '' + /a/i === '/a/i')
    case "[object String]":
      return "" + n == "" + r;
    case "[object Number]":
      return +n != +n ? +r != +r : +n == 0 ? 1 / +n === 1 / r : +n == +r;
    case "[object Date]":
    case "[object Boolean]":
      return +n == +r;
    case "[object Symbol]":
      return In.valueOf.call(n) === In.valueOf.call(r);
    case "[object ArrayBuffer]":
    case Dn:
      return nr(Tn(n), Tn(r), t, e);
  }
  var u = i === "[object Array]";
  if (!u && kn(n)) {
    var f = U(n);
    if (f !== U(r)) return !1;
    if (n.buffer === r.buffer && n.byteOffset === r.byteOffset) return !0;
    u = !0;
  }
  if (!u) {
    if (typeof n != "object" || typeof r != "object") return !1;
    var a = n.constructor, c = r.constructor;
    if (a !== c && !(g(a) && a instanceof a && g(c) && c instanceof c) && "constructor" in n && "constructor" in r)
      return !1;
  }
  t = t || [], e = e || [];
  for (var l = t.length; l--; )
    if (t[l] === n) return e[l] === r;
  if (t.push(n), e.push(r), u) {
    if (l = n.length, l !== r.length) return !1;
    for (; l--; )
      if (!K(n[l], r[l], t, e)) return !1;
  } else {
    var s = v(n), h;
    if (l = s.length, v(r).length !== l) return !1;
    for (; l--; )
      if (h = s[l], !(O(r, h) && K(n[h], r[h], t, e))) return !1;
  }
  return t.pop(), e.pop(), !0;
}
function ut(n, r) {
  return K(n, r);
}
function $(n) {
  if (!I(n)) return [];
  var r = [];
  for (var t in n) r.push(t);
  return Ln && bn(n, r), r;
}
function fn(n) {
  var r = m(n);
  return function(t) {
    if (t == null) return !1;
    var e = $(t);
    if (m(e)) return !1;
    for (var i = 0; i < r; i++)
      if (!g(t[n[i]])) return !1;
    return n !== er || !g(t[an]);
  };
}
var an = "forEach", rr = "has", ln = ["clear", "delete"], tr = ["get", rr, "set"], it = ln.concat(an, tr), er = ln.concat(tr), ft = ["add"].concat(ln, an, rr);
const at = en ? fn(it) : p("Map"), lt = en ? fn(er) : p("WeakMap"), ct = en ? fn(ft) : p("Set"), ot = p("WeakSet");
function P(n) {
  for (var r = v(n), t = r.length, e = Array(t), i = 0; i < t; i++)
    e[i] = n[r[i]];
  return e;
}
function st(n) {
  for (var r = v(n), t = r.length, e = Array(t), i = 0; i < t; i++)
    e[i] = [r[i], n[r[i]]];
  return e;
}
function ur(n) {
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
function cn(n, r) {
  return function(t) {
    var e = arguments.length;
    if (r && (t = Object(t)), e < 2 || t == null) return t;
    for (var i = 1; i < e; i++)
      for (var u = arguments[i], f = n(u), a = f.length, c = 0; c < a; c++) {
        var l = f[c];
        (!r || t[l] === void 0) && (t[l] = u[l]);
      }
    return t;
  };
}
const ir = cn($), C = cn(v), fr = cn($, !0);
function vt() {
  return function() {
  };
}
function ar(n) {
  if (!I(n)) return {};
  if (Sn) return Sn(n);
  var r = vt();
  r.prototype = n;
  var t = new r();
  return r.prototype = null, t;
}
function pt(n, r) {
  var t = ar(n);
  return r && C(t, r), t;
}
function ht(n) {
  return I(n) ? N(n) ? n.slice() : ir({}, n) : n;
}
function gt(n, r) {
  return r(n), n;
}
function lr(n) {
  return N(n) ? n : [n];
}
o.toPath = lr;
function q(n) {
  return o.toPath(n);
}
function on(n, r) {
  for (var t = r.length, e = 0; e < t; e++) {
    if (n == null) return;
    n = n[r[e]];
  }
  return t ? n : void 0;
}
function cr(n, r, t) {
  var e = on(n, q(r));
  return Jn(e) ? t : e;
}
function yt(n, r) {
  r = q(r);
  for (var t = r.length, e = 0; e < t; e++) {
    var i = r[e];
    if (!O(n, i)) return !1;
    n = n[i];
  }
  return !!t;
}
function sn(n) {
  return n;
}
function R(n) {
  return n = C({}, n), function(r) {
    return jn(r, n);
  };
}
function vn(n) {
  return n = q(n), function(r) {
    return on(r, n);
  };
}
function z(n, r, t) {
  if (r === void 0) return n;
  switch (t == null ? 3 : t) {
    case 1:
      return function(e) {
        return n.call(r, e);
      };
    // The 2-argument case is omitted because we’re not using it.
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
function or(n, r, t) {
  return n == null ? sn : g(n) ? z(n, r, t) : I(n) && !N(n) ? R(n) : vn(n);
}
function pn(n, r) {
  return or(n, r, 1 / 0);
}
o.iteratee = pn;
function d(n, r, t) {
  return o.iteratee !== pn ? o.iteratee(n, r) : or(n, r, t);
}
function mt(n, r, t) {
  r = d(r, t);
  for (var e = v(n), i = e.length, u = {}, f = 0; f < i; f++) {
    var a = e[f];
    u[a] = r(n[a], a, n);
  }
  return u;
}
function sr() {
}
function dt(n) {
  return n == null ? sr : function(r) {
    return cr(n, r);
  };
}
function wt(n, r, t) {
  var e = Array(Math.max(0, n));
  r = z(r, t, 1);
  for (var i = 0; i < n; i++) e[i] = r(i);
  return e;
}
function k(n, r) {
  return r == null && (r = n, n = 0), n + Math.floor(Math.random() * (r - n + 1));
}
const V = Date.now || function() {
  return (/* @__PURE__ */ new Date()).getTime();
};
function vr(n) {
  var r = function(u) {
    return n[u];
  }, t = "(?:" + v(n).join("|") + ")", e = RegExp(t), i = RegExp(t, "g");
  return function(u) {
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
}, _t = vr(pr), At = ur(pr), Ot = vr(At), Mt = o.templateSettings = {
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
}, It = /\\|'|\r|\n|\u2028|\u2029/g;
function Nt(n) {
  return "\\" + Et[n];
}
var St = /^\s*(\w|\$)+\s*$/;
function Bt(n, r, t) {
  !r && t && (r = t), r = fr({}, r, o.templateSettings);
  var e = RegExp([
    (r.escape || G).source,
    (r.interpolate || G).source,
    (r.evaluate || G).source
  ].join("|") + "|$", "g"), i = 0, u = "__p+='";
  n.replace(e, function(l, s, h, dn, wn) {
    return u += n.slice(i, wn).replace(It, Nt), i = wn + l.length, s ? u += `'+
((__t=(` + s + `))==null?'':_.escape(__t))+
'` : h ? u += `'+
((__t=(` + h + `))==null?'':__t)+
'` : dn && (u += `';
` + dn + `
__p+='`), l;
  }), u += `';
`;
  var f = r.variable;
  if (f) {
    if (!St.test(f)) throw new Error(
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
function Pt(n, r, t) {
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
var Tt = 0;
function Dt(n) {
  var r = ++Tt + "";
  return n ? n + r : r;
}
function Rt(n) {
  var r = o(n);
  return r._chain = !0, r;
}
function hr(n, r, t, e, i) {
  if (!(e instanceof r)) return n.apply(t, i);
  var u = ar(n.prototype), f = n.apply(u, i);
  return I(f) ? f : u;
}
var T = y(function(n, r) {
  var t = T.placeholder, e = function() {
    for (var i = 0, u = r.length, f = Array(u), a = 0; a < u; a++)
      f[a] = r[a] === t ? arguments[i++] : r[a];
    for (; i < arguments.length; ) f.push(arguments[i++]);
    return hr(n, e, this, this, f);
  };
  return e;
});
T.placeholder = o;
const gr = y(function(n, r, t) {
  if (!g(n)) throw new TypeError("Bind must be called on a function");
  var e = y(function(i) {
    return hr(n, e, r, this, t.concat(i));
  });
  return e;
}), w = Kn(m);
function S(n, r, t, e) {
  if (e = e || [], !r && r !== 0)
    r = 1 / 0;
  else if (r <= 0)
    return e.concat(n);
  for (var i = e.length, u = 0, f = m(n); u < f; u++) {
    var a = n[u];
    if (w(a) && (N(a) || un(a)))
      if (r > 1)
        S(a, r - 1, t, e), i = e.length;
      else
        for (var c = 0, l = a.length; c < l; ) e[i++] = a[c++];
    else t || (e[i++] = a);
  }
  return e;
}
const Vt = y(function(n, r) {
  r = S(r, !1, !1);
  var t = r.length;
  if (t < 1) throw new Error("bindAll must be passed function names");
  for (; t--; ) {
    var e = r[t];
    n[e] = gr(n[e], n);
  }
  return n;
});
function Ft(n, r) {
  var t = function(e) {
    var i = t.cache, u = "" + (r ? r.apply(this, arguments) : e);
    return O(i, u) || (i[u] = n.apply(this, arguments)), i[u];
  };
  return t.cache = {}, t;
}
const yr = y(function(n, r, t) {
  return setTimeout(function() {
    return n.apply(null, t);
  }, r);
}), $t = T(yr, o, 1);
function qt(n, r, t) {
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
function zt(n, r, t) {
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
function Lt(n, r) {
  return T(r, n);
}
function hn(n) {
  return function() {
    return !n.apply(this, arguments);
  };
}
function Jt() {
  var n = arguments, r = n.length - 1;
  return function() {
    for (var t = r, e = n[r].apply(this, arguments); t--; ) e = n[t].call(this, e);
    return e;
  };
}
function Ut(n, r) {
  return function() {
    if (--n < 1)
      return r.apply(this, arguments);
  };
}
function mr(n, r) {
  var t;
  return function() {
    return --n > 0 && (t = r.apply(this, arguments)), n <= 1 && (r = null), t;
  };
}
const Ct = T(mr, 2);
function dr(n, r, t) {
  r = d(r, t);
  for (var e = v(n), i, u = 0, f = e.length; u < f; u++)
    if (i = e[u], r(n[i], i, n)) return i;
}
function wr(n) {
  return function(r, t, e) {
    t = d(t, e);
    for (var i = m(r), u = n > 0 ? 0 : i - 1; u >= 0 && u < i; u += n)
      if (t(r[u], u, r)) return u;
    return -1;
  };
}
const gn = wr(1), _r = wr(-1);
function Ar(n, r, t, e) {
  t = d(t, e, 1);
  for (var i = t(r), u = 0, f = m(n); u < f; ) {
    var a = Math.floor((u + f) / 2);
    t(n[a]) < i ? u = a + 1 : f = a;
  }
  return u;
}
function Or(n, r, t) {
  return function(e, i, u) {
    var f = 0, a = m(e);
    if (typeof u == "number")
      n > 0 ? f = u >= 0 ? u : Math.max(u + a, f) : a = u >= 0 ? Math.min(u + 1, a) : u + a + 1;
    else if (t && u && a)
      return u = t(e, i), e[u] === i ? u : -1;
    if (i !== i)
      return u = r(F.call(e, f, a), Qn), u >= 0 ? u + f : -1;
    for (u = n > 0 ? f : a - 1; u >= 0 && u < a; u += n)
      if (e[u] === i) return u;
    return -1;
  };
}
const Mr = Or(1, gn, Ar), Wt = Or(-1, _r);
function b(n, r, t) {
  var e = w(n) ? gn : dr, i = e(n, r, t);
  if (i !== void 0 && i !== -1) return n[i];
}
function Xt(n, r) {
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
function E(n, r, t) {
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
const H = Er(1), Rn = Er(-1);
function B(n, r, t) {
  var e = [];
  return r = d(r, t), A(n, function(i, u, f) {
    r(i, u, f) && e.push(i);
  }), e;
}
function Yt(n, r, t) {
  return B(n, hn(d(r)), t);
}
function Vn(n, r, t) {
  r = d(r, t);
  for (var e = !w(n) && v(n), i = (e || n).length, u = 0; u < i; u++) {
    var f = e ? e[u] : u;
    if (!r(n[f], f, n)) return !1;
  }
  return !0;
}
function Fn(n, r, t) {
  r = d(r, t);
  for (var e = !w(n) && v(n), i = (e || n).length, u = 0; u < i; u++) {
    var f = e ? e[u] : u;
    if (r(n[f], f, n)) return !0;
  }
  return !1;
}
function _(n, r, t, e) {
  return w(n) || (n = P(n)), (typeof t != "number" || e) && (t = 0), Mr(n, r, t) >= 0;
}
const Gt = y(function(n, r, t) {
  var e, i;
  return g(r) ? i = r : (r = q(r), e = r.slice(0, -1), r = r[r.length - 1]), E(n, function(u) {
    var f = i;
    if (!f) {
      if (e && e.length && (u = on(u, e)), u == null) return;
      f = u[r];
    }
    return f == null ? f : f.apply(u, t);
  });
});
function yn(n, r) {
  return E(n, vn(r));
}
function Ht(n, r) {
  return B(n, R(r));
}
function Ir(n, r, t) {
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
function Qt(n, r, t) {
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
var Zt = /[^\ud800-\udfff]|[\ud800-\udbff][\udc00-\udfff]|[\ud800-\udfff]/g;
function Nr(n) {
  return n ? N(n) ? F.call(n) : tn(n) ? n.match(Zt) : w(n) ? E(n, sn) : P(n) : [];
}
function Sr(n, r, t) {
  if (r == null || t)
    return w(n) || (n = P(n)), n[k(n.length - 1)];
  var e = Nr(n), i = m(e);
  r = Math.max(Math.min(r, i), 0);
  for (var u = i - 1, f = 0; f < r; f++) {
    var a = k(f, u), c = e[f];
    e[f] = e[a], e[a] = c;
  }
  return e.slice(0, r);
}
function Kt(n) {
  return Sr(n, 1 / 0);
}
function xt(n, r, t) {
  var e = 0;
  return r = d(r, t), yn(E(n, function(i, u, f) {
    return {
      value: i,
      index: e++,
      criteria: r(i, u, f)
    };
  }).sort(function(i, u) {
    var f = i.criteria, a = u.criteria;
    if (f !== a) {
      if (f > a || f === void 0) return 1;
      if (f < a || a === void 0) return -1;
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
const kt = Y(function(n, r, t) {
  O(n, t) ? n[t].push(r) : n[t] = [r];
}), bt = Y(function(n, r, t) {
  n[t] = r;
}), jt = Y(function(n, r, t) {
  O(n, t) ? n[t]++ : n[t] = 1;
}), ne = Y(function(n, r, t) {
  n[t ? 0 : 1].push(r);
}, !0);
function re(n) {
  return n == null ? 0 : w(n) ? n.length : v(n).length;
}
function te(n, r, t) {
  return r in t;
}
const Br = y(function(n, r) {
  var t = {}, e = r[0];
  if (n == null) return t;
  g(e) ? (r.length > 1 && (e = z(e, r[1])), r = $(n)) : (e = te, r = S(r, !1, !1), n = Object(n));
  for (var i = 0, u = r.length; i < u; i++) {
    var f = r[i], a = n[f];
    e(a, f, n) && (t[f] = a);
  }
  return t;
}), ee = y(function(n, r) {
  var t = r[0], e;
  return g(t) ? (t = hn(t), r.length > 1 && (e = r[1])) : (r = E(S(r, !1, !1), String), t = function(i, u) {
    return !_(r, u);
  }), Br(n, t, e);
});
function Pr(n, r, t) {
  return F.call(n, 0, Math.max(0, n.length - (r == null || t ? 1 : r)));
}
function Q(n, r, t) {
  return n == null || n.length < 1 ? r == null || t ? void 0 : [] : r == null || t ? n[0] : Pr(n, n.length - r);
}
function L(n, r, t) {
  return F.call(n, r == null || t ? 1 : r);
}
function ue(n, r, t) {
  return n == null || n.length < 1 ? r == null || t ? void 0 : [] : r == null || t ? n[n.length - 1] : L(n, Math.max(0, n.length - r));
}
function ie(n) {
  return B(n, Boolean);
}
function fe(n, r) {
  return S(n, r, !1);
}
const Tr = y(function(n, r) {
  return r = S(r, !0, !0), B(n, function(t) {
    return !_(r, t);
  });
}), ae = y(function(n, r) {
  return Tr(n, r);
});
function j(n, r, t, e) {
  Un(r) || (e = t, t = r, r = !1), t != null && (t = d(t, e));
  for (var i = [], u = [], f = 0, a = m(n); f < a; f++) {
    var c = n[f], l = t ? t(c, f, n) : c;
    r && !t ? ((!f || u !== l) && i.push(c), u = l) : t ? _(u, l) || (u.push(l), i.push(c)) : _(i, c) || i.push(c);
  }
  return i;
}
const le = y(function(n) {
  return j(S(n, !0, !0));
});
function ce(n) {
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
  for (var r = n && Ir(n, m).length || 0, t = Array(r), e = 0; e < r; e++)
    t[e] = yn(n, e);
  return t;
}
const oe = y(nn);
function se(n, r) {
  for (var t = {}, e = 0, i = m(n); e < i; e++)
    r ? t[n[e]] = r[e] : t[n[e][0]] = n[e][1];
  return t;
}
function ve(n, r, t) {
  r == null && (r = n || 0, n = 0), t || (t = r < n ? -1 : 1);
  for (var e = Math.max(Math.ceil((r - n) / t), 0), i = Array(e), u = 0; u < e; u++, n += t)
    i[u] = n;
  return i;
}
function pe(n, r) {
  if (r == null || r < 1) return [];
  for (var t = [], e = 0, i = n.length; e < i; )
    t.push(F.call(n, e, e += r));
  return t;
}
function mn(n, r) {
  return n._chain ? o(r).chain() : r;
}
function Dr(n) {
  return A(x(n), function(r) {
    var t = o[r] = n[r];
    o.prototype[r] = function() {
      var e = [this._wrapped];
      return zr.apply(e, arguments), mn(this, t.apply(o, e));
    };
  }), o;
}
A(["pop", "push", "reverse", "shift", "sort", "splice", "unshift"], function(n) {
  var r = X[n];
  o.prototype[n] = function() {
    var t = this._wrapped;
    return t != null && (r.apply(t, arguments), (n === "shift" || n === "splice") && t.length === 0 && delete t[0]), mn(this, t);
  };
});
A(["concat", "join", "slice"], function(n) {
  var r = X[n];
  o.prototype[n] = function() {
    var t = this._wrapped;
    return t != null && (t = r.apply(t, arguments)), mn(this, t);
  };
});
const he = /* @__PURE__ */ Object.freeze(/* @__PURE__ */ Object.defineProperty({
  __proto__: null,
  VERSION: qn,
  after: Ut,
  all: Vn,
  allKeys: $,
  any: Fn,
  assign: C,
  before: mr,
  bind: gr,
  bindAll: Vt,
  chain: Rt,
  chunk: pe,
  clone: ht,
  collect: E,
  compact: ie,
  compose: Jt,
  constant: Zn,
  contains: _,
  countBy: jt,
  create: pt,
  debounce: zt,
  default: o,
  defaults: fr,
  defer: $t,
  delay: yr,
  detect: b,
  difference: Tr,
  drop: L,
  each: A,
  escape: _t,
  every: Vn,
  extend: ir,
  extendOwn: C,
  filter: B,
  find: b,
  findIndex: gn,
  findKey: dr,
  findLastIndex: _r,
  findWhere: Xt,
  first: Q,
  flatten: fe,
  foldl: H,
  foldr: Rn,
  forEach: A,
  functions: x,
  get: cr,
  groupBy: kt,
  has: yt,
  head: Q,
  identity: sn,
  include: _,
  includes: _,
  indexBy: bt,
  indexOf: Mr,
  initial: Pr,
  inject: H,
  intersection: ce,
  invert: ur,
  invoke: Gt,
  isArguments: un,
  isArray: N,
  isArrayBuffer: Xn,
  isBoolean: Un,
  isDataView: J,
  isDate: Hr,
  isElement: Gr,
  isEmpty: et,
  isEqual: ut,
  isError: Zr,
  isFinite: br,
  isFunction: g,
  isMap: at,
  isMatch: jn,
  isNaN: Qn,
  isNull: Yr,
  isNumber: Cn,
  isObject: I,
  isRegExp: Qr,
  isSet: ct,
  isString: tn,
  isSymbol: Wn,
  isTypedArray: kn,
  isUndefined: Jn,
  isWeakMap: lt,
  isWeakSet: ot,
  iteratee: pn,
  keys: v,
  last: ue,
  lastIndexOf: Wt,
  map: E,
  mapObject: mt,
  matcher: R,
  matches: R,
  max: Ir,
  memoize: Ft,
  methods: x,
  min: Qt,
  mixin: Dr,
  negate: hn,
  noop: sr,
  now: V,
  object: se,
  omit: ee,
  once: Ct,
  pairs: st,
  partial: T,
  partition: ne,
  pick: Br,
  pluck: yn,
  property: vn,
  propertyOf: dt,
  random: k,
  range: ve,
  reduce: H,
  reduceRight: Rn,
  reject: Yt,
  rest: L,
  restArguments: y,
  result: Pt,
  sample: Sr,
  select: B,
  shuffle: Kt,
  size: re,
  some: Fn,
  sortBy: xt,
  sortedIndex: Ar,
  tail: L,
  take: Q,
  tap: gt,
  template: Bt,
  templateSettings: Mt,
  throttle: qt,
  times: wt,
  toArray: Nr,
  toPath: lr,
  transpose: nn,
  unescape: Ot,
  union: le,
  uniq: j,
  unique: j,
  uniqueId: Dt,
  unzip: nn,
  values: P,
  where: Ht,
  without: ae,
  wrap: Lt,
  zip: oe
}, Symbol.toStringTag, { value: "Module" }));
var M = Dr(he);
M._ = M;
const $n = {
  discountApplicationStrategy: "First",
  discounts: []
};
function ge(n) {
  const r = JSON.parse(
    M.get(n, ["discountNode", "metafield", "value"], "{}")
  );
  if (!JSON.parse(
    M.get(n, ["cart", "buyerIdentity", "customer", "metafield", "value"], "{}")
  )) return $n;
  const e = M.chain(n.cart.lines).sortBy((u) => u.quantity).map((u) => Mn(On({}, u), { id: M.escape(u.id) })).value();
  return M.reduce(e, (u, f) => u + f.quantity, 0) < 0 ? $n : {
    discountApplicationStrategy: "Maximum",
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
const ye = Javy.IO.readSync(0, W);
W = W.subarray(0, ye);
const me = new TextEncoder().encode(JSON.stringify(ge(JSON.parse(new TextDecoder().decode(W)))));
Javy.IO.writeSync(1, me);
