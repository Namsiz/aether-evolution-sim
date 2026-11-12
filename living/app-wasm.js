(()=>{var uc="170",Li={LEFT:0,MIDDLE:1,RIGHT:2,ROTATE:0,DOLLY:1,PAN:2},Di={ROTATE:0,PAN:1,DOLLY_PAN:2,DOLLY_ROTATE:3},gu=0,Wc=1,_u=2;var jl=1,dc=2,Yn=3,Dn=0,ke=1,en=2,di=0,es=1,pi=2,Xc=3,Yc=4,xu=5,wi=100,yu=101,vu=102,Mu=103,Su=104,bu=200,Eu=201,Tu=202,wu=203,Ko=204,Jo=205,Au=206,Cu=207,Ru=208,Pu=209,Iu=210,Lu=211,Du=212,Uu=213,Nu=214,jo=0,Qo=1,ta=2,rs=3,ea=4,na=5,ia=6,sa=7,fc=0,Fu=1,Ou=2,fi=0,Bu=1,zu=2,ku=3,pc=4,Hu=5,Vu=6,Gu=7;var Ql=300,os=301,as=302,ra=303,oa=304,no=306,Un=1e3,Ln=1001,aa=1002,fn=1003,Wu=1004;var js=1005;var je=1006,vo=1007;var qn=1008;var Jn=1009,th=1010,eh=1011,Bs=1012,mc=1013,Ci=1014,_n=1015,Ys=1016,gc=1017,_c=1018,cs=1020,nh=35902,ih=1021,sh=1022,dn=1023,rh=1024,oh=1025,ns=1026,ls=1027,qs=1028,xc=1029,ah=1030,yc=1031;var vc=1033,wr=33776,Ar=33777,Cr=33778,Rr=33779,ca=35840,la=35841,ha=35842,ua=35843,da=36196,fa=37492,pa=37496,ma=37808,ga=37809,_a=37810,xa=37811,ya=37812,va=37813,Ma=37814,Sa=37815,ba=37816,Ea=37817,Ta=37818,wa=37819,Aa=37820,Ca=37821,Pr=36492,Ra=36494,Pa=36495,ch=36283,Ia=36284,La=36285,Da=36286;var Ir=2300,Ua=2301,Mo=2302,qc=2400,$c=2401,Zc=2402;var Xu=3200,Yu=3201;var Mc=0,qu=1,hi="",we="srgb",_s="srgb-linear",io="linear",xe="srgb";var Ni=7680;var Kc=519,$u=512,Zu=513,Ku=514,lh=515,Ju=516,ju=517,Qu=518,td=519,Na=35044,so=35048;var Jc="300 es",$n=2e3,Lr=2001,jn=class{addEventListener(t,e){this._listeners===void 0&&(this._listeners={});let n=this._listeners;n[t]===void 0&&(n[t]=[]),n[t].indexOf(e)===-1&&n[t].push(e)}hasEventListener(t,e){if(this._listeners===void 0)return!1;let n=this._listeners;return n[t]!==void 0&&n[t].indexOf(e)!==-1}removeEventListener(t,e){if(this._listeners===void 0)return;let s=this._listeners[t];if(s!==void 0){let r=s.indexOf(e);r!==-1&&s.splice(r,1)}}dispatchEvent(t){if(this._listeners===void 0)return;let n=this._listeners[t.type];if(n!==void 0){t.target=this;let s=n.slice(0);for(let r=0,o=s.length;r<o;r++)s[r].call(this,t);t.target=null}}},We=["00","01","02","03","04","05","06","07","08","09","0a","0b","0c","0d","0e","0f","10","11","12","13","14","15","16","17","18","19","1a","1b","1c","1d","1e","1f","20","21","22","23","24","25","26","27","28","29","2a","2b","2c","2d","2e","2f","30","31","32","33","34","35","36","37","38","39","3a","3b","3c","3d","3e","3f","40","41","42","43","44","45","46","47","48","49","4a","4b","4c","4d","4e","4f","50","51","52","53","54","55","56","57","58","59","5a","5b","5c","5d","5e","5f","60","61","62","63","64","65","66","67","68","69","6a","6b","6c","6d","6e","6f","70","71","72","73","74","75","76","77","78","79","7a","7b","7c","7d","7e","7f","80","81","82","83","84","85","86","87","88","89","8a","8b","8c","8d","8e","8f","90","91","92","93","94","95","96","97","98","99","9a","9b","9c","9d","9e","9f","a0","a1","a2","a3","a4","a5","a6","a7","a8","a9","aa","ab","ac","ad","ae","af","b0","b1","b2","b3","b4","b5","b6","b7","b8","b9","ba","bb","bc","bd","be","bf","c0","c1","c2","c3","c4","c5","c6","c7","c8","c9","ca","cb","cc","cd","ce","cf","d0","d1","d2","d3","d4","d5","d6","d7","d8","d9","da","db","dc","dd","de","df","e0","e1","e2","e3","e4","e5","e6","e7","e8","e9","ea","eb","ec","ed","ee","ef","f0","f1","f2","f3","f4","f5","f6","f7","f8","f9","fa","fb","fc","fd","fe","ff"],jc=1234567,Ns=Math.PI/180,zs=180/Math.PI;function Zn(){let i=Math.random()*4294967295|0,t=Math.random()*4294967295|0,e=Math.random()*4294967295|0,n=Math.random()*4294967295|0;return(We[i&255]+We[i>>8&255]+We[i>>16&255]+We[i>>24&255]+"-"+We[t&255]+We[t>>8&255]+"-"+We[t>>16&15|64]+We[t>>24&255]+"-"+We[e&63|128]+We[e>>8&255]+"-"+We[e>>16&255]+We[e>>24&255]+We[n&255]+We[n>>8&255]+We[n>>16&255]+We[n>>24&255]).toLowerCase()}function Ye(i,t,e){return Math.max(t,Math.min(e,i))}function Sc(i,t){return(i%t+t)%t}function ed(i,t,e,n,s){return n+(i-t)*(s-n)/(e-t)}function nd(i,t,e){return i!==t?(e-i)/(t-i):0}function Fs(i,t,e){return(1-e)*i+e*t}function id(i,t,e,n){return Fs(i,t,1-Math.exp(-e*n))}function sd(i,t=1){return t-Math.abs(Sc(i,t*2)-t)}function rd(i,t,e){return i<=t?0:i>=e?1:(i=(i-t)/(e-t),i*i*(3-2*i))}function od(i,t,e){return i<=t?0:i>=e?1:(i=(i-t)/(e-t),i*i*i*(i*(i*6-15)+10))}function ad(i,t){return i+Math.floor(Math.random()*(t-i+1))}function cd(i,t){return i+Math.random()*(t-i)}function ld(i){return i*(.5-Math.random())}function hd(i){i!==void 0&&(jc=i);let t=jc+=1831565813;return t=Math.imul(t^t>>>15,t|1),t^=t+Math.imul(t^t>>>7,t|61),((t^t>>>14)>>>0)/4294967296}function ud(i){return i*Ns}function dd(i){return i*zs}function fd(i){return(i&i-1)===0&&i!==0}function pd(i){return Math.pow(2,Math.ceil(Math.log(i)/Math.LN2))}function md(i){return Math.pow(2,Math.floor(Math.log(i)/Math.LN2))}function gd(i,t,e,n,s){let r=Math.cos,o=Math.sin,a=r(e/2),c=o(e/2),l=r((t+n)/2),h=o((t+n)/2),u=r((t-n)/2),p=o((t-n)/2),f=r((n-t)/2),g=o((n-t)/2);switch(s){case"XYX":i.set(a*h,c*u,c*p,a*l);break;case"YZY":i.set(c*p,a*h,c*u,a*l);break;case"ZXZ":i.set(c*u,c*p,a*h,a*l);break;case"XZX":i.set(a*h,c*g,c*f,a*l);break;case"YXY":i.set(c*f,a*h,c*g,a*l);break;case"ZYZ":i.set(c*g,c*f,a*h,a*l);break;default:console.warn("THREE.MathUtils: .setQuaternionFromProperEuler() encountered an unknown order: "+s)}}function bn(i,t){switch(t.constructor){case Float32Array:return i;case Uint32Array:return i/4294967295;case Uint16Array:return i/65535;case Uint8Array:return i/255;case Int32Array:return Math.max(i/2147483647,-1);case Int16Array:return Math.max(i/32767,-1);case Int8Array:return Math.max(i/127,-1);default:throw new Error("Invalid component type.")}}function ge(i,t){switch(t.constructor){case Float32Array:return i;case Uint32Array:return Math.round(i*4294967295);case Uint16Array:return Math.round(i*65535);case Uint8Array:return Math.round(i*255);case Int32Array:return Math.round(i*2147483647);case Int16Array:return Math.round(i*32767);case Int8Array:return Math.round(i*127);default:throw new Error("Invalid component type.")}}var $s={DEG2RAD:Ns,RAD2DEG:zs,generateUUID:Zn,clamp:Ye,euclideanModulo:Sc,mapLinear:ed,inverseLerp:nd,lerp:Fs,damp:id,pingpong:sd,smoothstep:rd,smootherstep:od,randInt:ad,randFloat:cd,randFloatSpread:ld,seededRandom:hd,degToRad:ud,radToDeg:dd,isPowerOfTwo:fd,ceilPowerOfTwo:pd,floorPowerOfTwo:md,setQuaternionFromProperEuler:gd,normalize:ge,denormalize:bn},Wt=class i{constructor(t=0,e=0){i.prototype.isVector2=!0,this.x=t,this.y=e}get width(){return this.x}set width(t){this.x=t}get height(){return this.y}set height(t){this.y=t}set(t,e){return this.x=t,this.y=e,this}setScalar(t){return this.x=t,this.y=t,this}setX(t){return this.x=t,this}setY(t){return this.y=t,this}setComponent(t,e){switch(t){case 0:this.x=e;break;case 1:this.y=e;break;default:throw new Error("index is out of range: "+t)}return this}getComponent(t){switch(t){case 0:return this.x;case 1:return this.y;default:throw new Error("index is out of range: "+t)}}clone(){return new this.constructor(this.x,this.y)}copy(t){return this.x=t.x,this.y=t.y,this}add(t){return this.x+=t.x,this.y+=t.y,this}addScalar(t){return this.x+=t,this.y+=t,this}addVectors(t,e){return this.x=t.x+e.x,this.y=t.y+e.y,this}addScaledVector(t,e){return this.x+=t.x*e,this.y+=t.y*e,this}sub(t){return this.x-=t.x,this.y-=t.y,this}subScalar(t){return this.x-=t,this.y-=t,this}subVectors(t,e){return this.x=t.x-e.x,this.y=t.y-e.y,this}multiply(t){return this.x*=t.x,this.y*=t.y,this}multiplyScalar(t){return this.x*=t,this.y*=t,this}divide(t){return this.x/=t.x,this.y/=t.y,this}divideScalar(t){return this.multiplyScalar(1/t)}applyMatrix3(t){let e=this.x,n=this.y,s=t.elements;return this.x=s[0]*e+s[3]*n+s[6],this.y=s[1]*e+s[4]*n+s[7],this}min(t){return this.x=Math.min(this.x,t.x),this.y=Math.min(this.y,t.y),this}max(t){return this.x=Math.max(this.x,t.x),this.y=Math.max(this.y,t.y),this}clamp(t,e){return this.x=Math.max(t.x,Math.min(e.x,this.x)),this.y=Math.max(t.y,Math.min(e.y,this.y)),this}clampScalar(t,e){return this.x=Math.max(t,Math.min(e,this.x)),this.y=Math.max(t,Math.min(e,this.y)),this}clampLength(t,e){let n=this.length();return this.divideScalar(n||1).multiplyScalar(Math.max(t,Math.min(e,n)))}floor(){return this.x=Math.floor(this.x),this.y=Math.floor(this.y),this}ceil(){return this.x=Math.ceil(this.x),this.y=Math.ceil(this.y),this}round(){return this.x=Math.round(this.x),this.y=Math.round(this.y),this}roundToZero(){return this.x=Math.trunc(this.x),this.y=Math.trunc(this.y),this}negate(){return this.x=-this.x,this.y=-this.y,this}dot(t){return this.x*t.x+this.y*t.y}cross(t){return this.x*t.y-this.y*t.x}lengthSq(){return this.x*this.x+this.y*this.y}length(){return Math.sqrt(this.x*this.x+this.y*this.y)}manhattanLength(){return Math.abs(this.x)+Math.abs(this.y)}normalize(){return this.divideScalar(this.length()||1)}angle(){return Math.atan2(-this.y,-this.x)+Math.PI}angleTo(t){let e=Math.sqrt(this.lengthSq()*t.lengthSq());if(e===0)return Math.PI/2;let n=this.dot(t)/e;return Math.acos(Ye(n,-1,1))}distanceTo(t){return Math.sqrt(this.distanceToSquared(t))}distanceToSquared(t){let e=this.x-t.x,n=this.y-t.y;return e*e+n*n}manhattanDistanceTo(t){return Math.abs(this.x-t.x)+Math.abs(this.y-t.y)}setLength(t){return this.normalize().multiplyScalar(t)}lerp(t,e){return this.x+=(t.x-this.x)*e,this.y+=(t.y-this.y)*e,this}lerpVectors(t,e,n){return this.x=t.x+(e.x-t.x)*n,this.y=t.y+(e.y-t.y)*n,this}equals(t){return t.x===this.x&&t.y===this.y}fromArray(t,e=0){return this.x=t[e],this.y=t[e+1],this}toArray(t=[],e=0){return t[e]=this.x,t[e+1]=this.y,t}fromBufferAttribute(t,e){return this.x=t.getX(e),this.y=t.getY(e),this}rotateAround(t,e){let n=Math.cos(e),s=Math.sin(e),r=this.x-t.x,o=this.y-t.y;return this.x=r*n-o*s+t.x,this.y=r*s+o*n+t.y,this}random(){return this.x=Math.random(),this.y=Math.random(),this}*[Symbol.iterator](){yield this.x,yield this.y}},ee=class i{constructor(t,e,n,s,r,o,a,c,l){i.prototype.isMatrix3=!0,this.elements=[1,0,0,0,1,0,0,0,1],t!==void 0&&this.set(t,e,n,s,r,o,a,c,l)}set(t,e,n,s,r,o,a,c,l){let h=this.elements;return h[0]=t,h[1]=s,h[2]=a,h[3]=e,h[4]=r,h[5]=c,h[6]=n,h[7]=o,h[8]=l,this}identity(){return this.set(1,0,0,0,1,0,0,0,1),this}copy(t){let e=this.elements,n=t.elements;return e[0]=n[0],e[1]=n[1],e[2]=n[2],e[3]=n[3],e[4]=n[4],e[5]=n[5],e[6]=n[6],e[7]=n[7],e[8]=n[8],this}extractBasis(t,e,n){return t.setFromMatrix3Column(this,0),e.setFromMatrix3Column(this,1),n.setFromMatrix3Column(this,2),this}setFromMatrix4(t){let e=t.elements;return this.set(e[0],e[4],e[8],e[1],e[5],e[9],e[2],e[6],e[10]),this}multiply(t){return this.multiplyMatrices(this,t)}premultiply(t){return this.multiplyMatrices(t,this)}multiplyMatrices(t,e){let n=t.elements,s=e.elements,r=this.elements,o=n[0],a=n[3],c=n[6],l=n[1],h=n[4],u=n[7],p=n[2],f=n[5],g=n[8],y=s[0],m=s[3],d=s[6],x=s[1],_=s[4],v=s[7],C=s[2],w=s[5],A=s[8];return r[0]=o*y+a*x+c*C,r[3]=o*m+a*_+c*w,r[6]=o*d+a*v+c*A,r[1]=l*y+h*x+u*C,r[4]=l*m+h*_+u*w,r[7]=l*d+h*v+u*A,r[2]=p*y+f*x+g*C,r[5]=p*m+f*_+g*w,r[8]=p*d+f*v+g*A,this}multiplyScalar(t){let e=this.elements;return e[0]*=t,e[3]*=t,e[6]*=t,e[1]*=t,e[4]*=t,e[7]*=t,e[2]*=t,e[5]*=t,e[8]*=t,this}determinant(){let t=this.elements,e=t[0],n=t[1],s=t[2],r=t[3],o=t[4],a=t[5],c=t[6],l=t[7],h=t[8];return e*o*h-e*a*l-n*r*h+n*a*c+s*r*l-s*o*c}invert(){let t=this.elements,e=t[0],n=t[1],s=t[2],r=t[3],o=t[4],a=t[5],c=t[6],l=t[7],h=t[8],u=h*o-a*l,p=a*c-h*r,f=l*r-o*c,g=e*u+n*p+s*f;if(g===0)return this.set(0,0,0,0,0,0,0,0,0);let y=1/g;return t[0]=u*y,t[1]=(s*l-h*n)*y,t[2]=(a*n-s*o)*y,t[3]=p*y,t[4]=(h*e-s*c)*y,t[5]=(s*r-a*e)*y,t[6]=f*y,t[7]=(n*c-l*e)*y,t[8]=(o*e-n*r)*y,this}transpose(){let t,e=this.elements;return t=e[1],e[1]=e[3],e[3]=t,t=e[2],e[2]=e[6],e[6]=t,t=e[5],e[5]=e[7],e[7]=t,this}getNormalMatrix(t){return this.setFromMatrix4(t).invert().transpose()}transposeIntoArray(t){let e=this.elements;return t[0]=e[0],t[1]=e[3],t[2]=e[6],t[3]=e[1],t[4]=e[4],t[5]=e[7],t[6]=e[2],t[7]=e[5],t[8]=e[8],this}setUvTransform(t,e,n,s,r,o,a){let c=Math.cos(r),l=Math.sin(r);return this.set(n*c,n*l,-n*(c*o+l*a)+o+t,-s*l,s*c,-s*(-l*o+c*a)+a+e,0,0,1),this}scale(t,e){return this.premultiply(So.makeScale(t,e)),this}rotate(t){return this.premultiply(So.makeRotation(-t)),this}translate(t,e){return this.premultiply(So.makeTranslation(t,e)),this}makeTranslation(t,e){return t.isVector2?this.set(1,0,t.x,0,1,t.y,0,0,1):this.set(1,0,t,0,1,e,0,0,1),this}makeRotation(t){let e=Math.cos(t),n=Math.sin(t);return this.set(e,-n,0,n,e,0,0,0,1),this}makeScale(t,e){return this.set(t,0,0,0,e,0,0,0,1),this}equals(t){let e=this.elements,n=t.elements;for(let s=0;s<9;s++)if(e[s]!==n[s])return!1;return!0}fromArray(t,e=0){for(let n=0;n<9;n++)this.elements[n]=t[n+e];return this}toArray(t=[],e=0){let n=this.elements;return t[e]=n[0],t[e+1]=n[1],t[e+2]=n[2],t[e+3]=n[3],t[e+4]=n[4],t[e+5]=n[5],t[e+6]=n[6],t[e+7]=n[7],t[e+8]=n[8],t}clone(){return new this.constructor().fromArray(this.elements)}},So=new ee;function hh(i){for(let t=i.length-1;t>=0;--t)if(i[t]>=65535)return!0;return!1}function Dr(i){return document.createElementNS("http://www.w3.org/1999/xhtml",i)}function _d(){let i=Dr("canvas");return i.style.display="block",i}var Qc={};function Ds(i){i in Qc||(Qc[i]=!0,console.warn(i))}function xd(i,t,e){return new Promise(function(n,s){function r(){switch(i.clientWaitSync(t,i.SYNC_FLUSH_COMMANDS_BIT,0)){case i.WAIT_FAILED:s();break;case i.TIMEOUT_EXPIRED:setTimeout(r,e);break;default:n()}}setTimeout(r,e)})}function yd(i){let t=i.elements;t[2]=.5*t[2]+.5*t[3],t[6]=.5*t[6]+.5*t[7],t[10]=.5*t[10]+.5*t[11],t[14]=.5*t[14]+.5*t[15]}function vd(i){let t=i.elements;t[11]===-1?(t[10]=-t[10]-1,t[14]=-t[14]):(t[10]=-t[10],t[14]=-t[14]+1)}var he={enabled:!0,workingColorSpace:_s,spaces:{},convert:function(i,t,e){return this.enabled===!1||t===e||!t||!e||(this.spaces[t].transfer===xe&&(i.r=Kn(i.r),i.g=Kn(i.g),i.b=Kn(i.b)),this.spaces[t].primaries!==this.spaces[e].primaries&&(i.applyMatrix3(this.spaces[t].toXYZ),i.applyMatrix3(this.spaces[e].fromXYZ)),this.spaces[e].transfer===xe&&(i.r=is(i.r),i.g=is(i.g),i.b=is(i.b))),i},fromWorkingColorSpace:function(i,t){return this.convert(i,this.workingColorSpace,t)},toWorkingColorSpace:function(i,t){return this.convert(i,t,this.workingColorSpace)},getPrimaries:function(i){return this.spaces[i].primaries},getTransfer:function(i){return i===hi?io:this.spaces[i].transfer},getLuminanceCoefficients:function(i,t=this.workingColorSpace){return i.fromArray(this.spaces[t].luminanceCoefficients)},define:function(i){Object.assign(this.spaces,i)},_getMatrix:function(i,t,e){return i.copy(this.spaces[t].toXYZ).multiply(this.spaces[e].fromXYZ)},_getDrawingBufferColorSpace:function(i){return this.spaces[i].outputColorSpaceConfig.drawingBufferColorSpace},_getUnpackColorSpace:function(i=this.workingColorSpace){return this.spaces[i].workingColorSpaceConfig.unpackColorSpace}};function Kn(i){return i<.04045?i*.0773993808:Math.pow(i*.9478672986+.0521327014,2.4)}function is(i){return i<.0031308?i*12.92:1.055*Math.pow(i,.41666)-.055}var tl=[.64,.33,.3,.6,.15,.06],el=[.2126,.7152,.0722],nl=[.3127,.329],il=new ee().set(.4123908,.3575843,.1804808,.212639,.7151687,.0721923,.0193308,.1191948,.9505322),sl=new ee().set(3.2409699,-1.5373832,-.4986108,-.9692436,1.8759675,.0415551,.0556301,-.203977,1.0569715);he.define({[_s]:{primaries:tl,whitePoint:nl,transfer:io,toXYZ:il,fromXYZ:sl,luminanceCoefficients:el,workingColorSpaceConfig:{unpackColorSpace:we},outputColorSpaceConfig:{drawingBufferColorSpace:we}},[we]:{primaries:tl,whitePoint:nl,transfer:xe,toXYZ:il,fromXYZ:sl,luminanceCoefficients:el,outputColorSpaceConfig:{drawingBufferColorSpace:we}}});var Fi,Fa=class{static getDataURL(t){if(/^data:/i.test(t.src)||typeof HTMLCanvasElement>"u")return t.src;let e;if(t instanceof HTMLCanvasElement)e=t;else{Fi===void 0&&(Fi=Dr("canvas")),Fi.width=t.width,Fi.height=t.height;let n=Fi.getContext("2d");t instanceof ImageData?n.putImageData(t,0,0):n.drawImage(t,0,0,t.width,t.height),e=Fi}return e.width>2048||e.height>2048?(console.warn("THREE.ImageUtils.getDataURL: Image converted to jpg for performance reasons",t),e.toDataURL("image/jpeg",.6)):e.toDataURL("image/png")}static sRGBToLinear(t){if(typeof HTMLImageElement<"u"&&t instanceof HTMLImageElement||typeof HTMLCanvasElement<"u"&&t instanceof HTMLCanvasElement||typeof ImageBitmap<"u"&&t instanceof ImageBitmap){let e=Dr("canvas");e.width=t.width,e.height=t.height;let n=e.getContext("2d");n.drawImage(t,0,0,t.width,t.height);let s=n.getImageData(0,0,t.width,t.height),r=s.data;for(let o=0;o<r.length;o++)r[o]=Kn(r[o]/255)*255;return n.putImageData(s,0,0),e}else if(t.data){let e=t.data.slice(0);for(let n=0;n<e.length;n++)e instanceof Uint8Array||e instanceof Uint8ClampedArray?e[n]=Math.floor(Kn(e[n]/255)*255):e[n]=Kn(e[n]);return{data:e,width:t.width,height:t.height}}else return console.warn("THREE.ImageUtils.sRGBToLinear(): Unsupported image type. No color space conversion applied."),t}},Md=0,Ur=class{constructor(t=null){this.isSource=!0,Object.defineProperty(this,"id",{value:Md++}),this.uuid=Zn(),this.data=t,this.dataReady=!0,this.version=0}set needsUpdate(t){t===!0&&this.version++}toJSON(t){let e=t===void 0||typeof t=="string";if(!e&&t.images[this.uuid]!==void 0)return t.images[this.uuid];let n={uuid:this.uuid,url:""},s=this.data;if(s!==null){let r;if(Array.isArray(s)){r=[];for(let o=0,a=s.length;o<a;o++)s[o].isDataTexture?r.push(bo(s[o].image)):r.push(bo(s[o]))}else r=bo(s);n.url=r}return e||(t.images[this.uuid]=n),n}};function bo(i){return typeof HTMLImageElement<"u"&&i instanceof HTMLImageElement||typeof HTMLCanvasElement<"u"&&i instanceof HTMLCanvasElement||typeof ImageBitmap<"u"&&i instanceof ImageBitmap?Fa.getDataURL(i):i.data?{data:Array.from(i.data),width:i.width,height:i.height,type:i.data.constructor.name}:(console.warn("THREE.Texture: Unable to serialize Texture."),{})}var Sd=0,nn=class i extends jn{constructor(t=i.DEFAULT_IMAGE,e=i.DEFAULT_MAPPING,n=Ln,s=Ln,r=je,o=qn,a=dn,c=Jn,l=i.DEFAULT_ANISOTROPY,h=hi){super(),this.isTexture=!0,Object.defineProperty(this,"id",{value:Sd++}),this.uuid=Zn(),this.name="",this.source=new Ur(t),this.mipmaps=[],this.mapping=e,this.channel=0,this.wrapS=n,this.wrapT=s,this.magFilter=r,this.minFilter=o,this.anisotropy=l,this.format=a,this.internalFormat=null,this.type=c,this.offset=new Wt(0,0),this.repeat=new Wt(1,1),this.center=new Wt(0,0),this.rotation=0,this.matrixAutoUpdate=!0,this.matrix=new ee,this.generateMipmaps=!0,this.premultiplyAlpha=!1,this.flipY=!0,this.unpackAlignment=4,this.colorSpace=h,this.userData={},this.version=0,this.onUpdate=null,this.isRenderTargetTexture=!1,this.pmremVersion=0}get image(){return this.source.data}set image(t=null){this.source.data=t}updateMatrix(){this.matrix.setUvTransform(this.offset.x,this.offset.y,this.repeat.x,this.repeat.y,this.rotation,this.center.x,this.center.y)}clone(){return new this.constructor().copy(this)}copy(t){return this.name=t.name,this.source=t.source,this.mipmaps=t.mipmaps.slice(0),this.mapping=t.mapping,this.channel=t.channel,this.wrapS=t.wrapS,this.wrapT=t.wrapT,this.magFilter=t.magFilter,this.minFilter=t.minFilter,this.anisotropy=t.anisotropy,this.format=t.format,this.internalFormat=t.internalFormat,this.type=t.type,this.offset.copy(t.offset),this.repeat.copy(t.repeat),this.center.copy(t.center),this.rotation=t.rotation,this.matrixAutoUpdate=t.matrixAutoUpdate,this.matrix.copy(t.matrix),this.generateMipmaps=t.generateMipmaps,this.premultiplyAlpha=t.premultiplyAlpha,this.flipY=t.flipY,this.unpackAlignment=t.unpackAlignment,this.colorSpace=t.colorSpace,this.userData=JSON.parse(JSON.stringify(t.userData)),this.needsUpdate=!0,this}toJSON(t){let e=t===void 0||typeof t=="string";if(!e&&t.textures[this.uuid]!==void 0)return t.textures[this.uuid];let n={metadata:{version:4.6,type:"Texture",generator:"Texture.toJSON"},uuid:this.uuid,name:this.name,image:this.source.toJSON(t).uuid,mapping:this.mapping,channel:this.channel,repeat:[this.repeat.x,this.repeat.y],offset:[this.offset.x,this.offset.y],center:[this.center.x,this.center.y],rotation:this.rotation,wrap:[this.wrapS,this.wrapT],format:this.format,internalFormat:this.internalFormat,type:this.type,colorSpace:this.colorSpace,minFilter:this.minFilter,magFilter:this.magFilter,anisotropy:this.anisotropy,flipY:this.flipY,generateMipmaps:this.generateMipmaps,premultiplyAlpha:this.premultiplyAlpha,unpackAlignment:this.unpackAlignment};return Object.keys(this.userData).length>0&&(n.userData=this.userData),e||(t.textures[this.uuid]=n),n}dispose(){this.dispatchEvent({type:"dispose"})}transformUv(t){if(this.mapping!==Ql)return t;if(t.applyMatrix3(this.matrix),t.x<0||t.x>1)switch(this.wrapS){case Un:t.x=t.x-Math.floor(t.x);break;case Ln:t.x=t.x<0?0:1;break;case aa:Math.abs(Math.floor(t.x)%2)===1?t.x=Math.ceil(t.x)-t.x:t.x=t.x-Math.floor(t.x);break}if(t.y<0||t.y>1)switch(this.wrapT){case Un:t.y=t.y-Math.floor(t.y);break;case Ln:t.y=t.y<0?0:1;break;case aa:Math.abs(Math.floor(t.y)%2)===1?t.y=Math.ceil(t.y)-t.y:t.y=t.y-Math.floor(t.y);break}return this.flipY&&(t.y=1-t.y),t}set needsUpdate(t){t===!0&&(this.version++,this.source.needsUpdate=!0)}set needsPMREMUpdate(t){t===!0&&this.pmremVersion++}};nn.DEFAULT_IMAGE=null;nn.DEFAULT_MAPPING=Ql;nn.DEFAULT_ANISOTROPY=1;var be=class i{constructor(t=0,e=0,n=0,s=1){i.prototype.isVector4=!0,this.x=t,this.y=e,this.z=n,this.w=s}get width(){return this.z}set width(t){this.z=t}get height(){return this.w}set height(t){this.w=t}set(t,e,n,s){return this.x=t,this.y=e,this.z=n,this.w=s,this}setScalar(t){return this.x=t,this.y=t,this.z=t,this.w=t,this}setX(t){return this.x=t,this}setY(t){return this.y=t,this}setZ(t){return this.z=t,this}setW(t){return this.w=t,this}setComponent(t,e){switch(t){case 0:this.x=e;break;case 1:this.y=e;break;case 2:this.z=e;break;case 3:this.w=e;break;default:throw new Error("index is out of range: "+t)}return this}getComponent(t){switch(t){case 0:return this.x;case 1:return this.y;case 2:return this.z;case 3:return this.w;default:throw new Error("index is out of range: "+t)}}clone(){return new this.constructor(this.x,this.y,this.z,this.w)}copy(t){return this.x=t.x,this.y=t.y,this.z=t.z,this.w=t.w!==void 0?t.w:1,this}add(t){return this.x+=t.x,this.y+=t.y,this.z+=t.z,this.w+=t.w,this}addScalar(t){return this.x+=t,this.y+=t,this.z+=t,this.w+=t,this}addVectors(t,e){return this.x=t.x+e.x,this.y=t.y+e.y,this.z=t.z+e.z,this.w=t.w+e.w,this}addScaledVector(t,e){return this.x+=t.x*e,this.y+=t.y*e,this.z+=t.z*e,this.w+=t.w*e,this}sub(t){return this.x-=t.x,this.y-=t.y,this.z-=t.z,this.w-=t.w,this}subScalar(t){return this.x-=t,this.y-=t,this.z-=t,this.w-=t,this}subVectors(t,e){return this.x=t.x-e.x,this.y=t.y-e.y,this.z=t.z-e.z,this.w=t.w-e.w,this}multiply(t){return this.x*=t.x,this.y*=t.y,this.z*=t.z,this.w*=t.w,this}multiplyScalar(t){return this.x*=t,this.y*=t,this.z*=t,this.w*=t,this}applyMatrix4(t){let e=this.x,n=this.y,s=this.z,r=this.w,o=t.elements;return this.x=o[0]*e+o[4]*n+o[8]*s+o[12]*r,this.y=o[1]*e+o[5]*n+o[9]*s+o[13]*r,this.z=o[2]*e+o[6]*n+o[10]*s+o[14]*r,this.w=o[3]*e+o[7]*n+o[11]*s+o[15]*r,this}divide(t){return this.x/=t.x,this.y/=t.y,this.z/=t.z,this.w/=t.w,this}divideScalar(t){return this.multiplyScalar(1/t)}setAxisAngleFromQuaternion(t){this.w=2*Math.acos(t.w);let e=Math.sqrt(1-t.w*t.w);return e<1e-4?(this.x=1,this.y=0,this.z=0):(this.x=t.x/e,this.y=t.y/e,this.z=t.z/e),this}setAxisAngleFromRotationMatrix(t){let e,n,s,r,c=t.elements,l=c[0],h=c[4],u=c[8],p=c[1],f=c[5],g=c[9],y=c[2],m=c[6],d=c[10];if(Math.abs(h-p)<.01&&Math.abs(u-y)<.01&&Math.abs(g-m)<.01){if(Math.abs(h+p)<.1&&Math.abs(u+y)<.1&&Math.abs(g+m)<.1&&Math.abs(l+f+d-3)<.1)return this.set(1,0,0,0),this;e=Math.PI;let _=(l+1)/2,v=(f+1)/2,C=(d+1)/2,w=(h+p)/4,A=(u+y)/4,P=(g+m)/4;return _>v&&_>C?_<.01?(n=0,s=.707106781,r=.707106781):(n=Math.sqrt(_),s=w/n,r=A/n):v>C?v<.01?(n=.707106781,s=0,r=.707106781):(s=Math.sqrt(v),n=w/s,r=P/s):C<.01?(n=.707106781,s=.707106781,r=0):(r=Math.sqrt(C),n=A/r,s=P/r),this.set(n,s,r,e),this}let x=Math.sqrt((m-g)*(m-g)+(u-y)*(u-y)+(p-h)*(p-h));return Math.abs(x)<.001&&(x=1),this.x=(m-g)/x,this.y=(u-y)/x,this.z=(p-h)/x,this.w=Math.acos((l+f+d-1)/2),this}setFromMatrixPosition(t){let e=t.elements;return this.x=e[12],this.y=e[13],this.z=e[14],this.w=e[15],this}min(t){return this.x=Math.min(this.x,t.x),this.y=Math.min(this.y,t.y),this.z=Math.min(this.z,t.z),this.w=Math.min(this.w,t.w),this}max(t){return this.x=Math.max(this.x,t.x),this.y=Math.max(this.y,t.y),this.z=Math.max(this.z,t.z),this.w=Math.max(this.w,t.w),this}clamp(t,e){return this.x=Math.max(t.x,Math.min(e.x,this.x)),this.y=Math.max(t.y,Math.min(e.y,this.y)),this.z=Math.max(t.z,Math.min(e.z,this.z)),this.w=Math.max(t.w,Math.min(e.w,this.w)),this}clampScalar(t,e){return this.x=Math.max(t,Math.min(e,this.x)),this.y=Math.max(t,Math.min(e,this.y)),this.z=Math.max(t,Math.min(e,this.z)),this.w=Math.max(t,Math.min(e,this.w)),this}clampLength(t,e){let n=this.length();return this.divideScalar(n||1).multiplyScalar(Math.max(t,Math.min(e,n)))}floor(){return this.x=Math.floor(this.x),this.y=Math.floor(this.y),this.z=Math.floor(this.z),this.w=Math.floor(this.w),this}ceil(){return this.x=Math.ceil(this.x),this.y=Math.ceil(this.y),this.z=Math.ceil(this.z),this.w=Math.ceil(this.w),this}round(){return this.x=Math.round(this.x),this.y=Math.round(this.y),this.z=Math.round(this.z),this.w=Math.round(this.w),this}roundToZero(){return this.x=Math.trunc(this.x),this.y=Math.trunc(this.y),this.z=Math.trunc(this.z),this.w=Math.trunc(this.w),this}negate(){return this.x=-this.x,this.y=-this.y,this.z=-this.z,this.w=-this.w,this}dot(t){return this.x*t.x+this.y*t.y+this.z*t.z+this.w*t.w}lengthSq(){return this.x*this.x+this.y*this.y+this.z*this.z+this.w*this.w}length(){return Math.sqrt(this.x*this.x+this.y*this.y+this.z*this.z+this.w*this.w)}manhattanLength(){return Math.abs(this.x)+Math.abs(this.y)+Math.abs(this.z)+Math.abs(this.w)}normalize(){return this.divideScalar(this.length()||1)}setLength(t){return this.normalize().multiplyScalar(t)}lerp(t,e){return this.x+=(t.x-this.x)*e,this.y+=(t.y-this.y)*e,this.z+=(t.z-this.z)*e,this.w+=(t.w-this.w)*e,this}lerpVectors(t,e,n){return this.x=t.x+(e.x-t.x)*n,this.y=t.y+(e.y-t.y)*n,this.z=t.z+(e.z-t.z)*n,this.w=t.w+(e.w-t.w)*n,this}equals(t){return t.x===this.x&&t.y===this.y&&t.z===this.z&&t.w===this.w}fromArray(t,e=0){return this.x=t[e],this.y=t[e+1],this.z=t[e+2],this.w=t[e+3],this}toArray(t=[],e=0){return t[e]=this.x,t[e+1]=this.y,t[e+2]=this.z,t[e+3]=this.w,t}fromBufferAttribute(t,e){return this.x=t.getX(e),this.y=t.getY(e),this.z=t.getZ(e),this.w=t.getW(e),this}random(){return this.x=Math.random(),this.y=Math.random(),this.z=Math.random(),this.w=Math.random(),this}*[Symbol.iterator](){yield this.x,yield this.y,yield this.z,yield this.w}},Oa=class extends jn{constructor(t=1,e=1,n={}){super(),this.isRenderTarget=!0,this.width=t,this.height=e,this.depth=1,this.scissor=new be(0,0,t,e),this.scissorTest=!1,this.viewport=new be(0,0,t,e);let s={width:t,height:e,depth:1};n=Object.assign({generateMipmaps:!1,internalFormat:null,minFilter:je,depthBuffer:!0,stencilBuffer:!1,resolveDepthBuffer:!0,resolveStencilBuffer:!0,depthTexture:null,samples:0,count:1},n);let r=new nn(s,n.mapping,n.wrapS,n.wrapT,n.magFilter,n.minFilter,n.format,n.type,n.anisotropy,n.colorSpace);r.flipY=!1,r.generateMipmaps=n.generateMipmaps,r.internalFormat=n.internalFormat,this.textures=[];let o=n.count;for(let a=0;a<o;a++)this.textures[a]=r.clone(),this.textures[a].isRenderTargetTexture=!0;this.depthBuffer=n.depthBuffer,this.stencilBuffer=n.stencilBuffer,this.resolveDepthBuffer=n.resolveDepthBuffer,this.resolveStencilBuffer=n.resolveStencilBuffer,this.depthTexture=n.depthTexture,this.samples=n.samples}get texture(){return this.textures[0]}set texture(t){this.textures[0]=t}setSize(t,e,n=1){if(this.width!==t||this.height!==e||this.depth!==n){this.width=t,this.height=e,this.depth=n;for(let s=0,r=this.textures.length;s<r;s++)this.textures[s].image.width=t,this.textures[s].image.height=e,this.textures[s].image.depth=n;this.dispose()}this.viewport.set(0,0,t,e),this.scissor.set(0,0,t,e)}clone(){return new this.constructor().copy(this)}copy(t){this.width=t.width,this.height=t.height,this.depth=t.depth,this.scissor.copy(t.scissor),this.scissorTest=t.scissorTest,this.viewport.copy(t.viewport),this.textures.length=0;for(let n=0,s=t.textures.length;n<s;n++)this.textures[n]=t.textures[n].clone(),this.textures[n].isRenderTargetTexture=!0;let e=Object.assign({},t.texture.image);return this.texture.source=new Ur(e),this.depthBuffer=t.depthBuffer,this.stencilBuffer=t.stencilBuffer,this.resolveDepthBuffer=t.resolveDepthBuffer,this.resolveStencilBuffer=t.resolveStencilBuffer,t.depthTexture!==null&&(this.depthTexture=t.depthTexture.clone()),this.samples=t.samples,this}dispose(){this.dispatchEvent({type:"dispose"})}},En=class extends Oa{constructor(t=1,e=1,n={}){super(t,e,n),this.isWebGLRenderTarget=!0}},Nr=class extends nn{constructor(t=null,e=1,n=1,s=1){super(null),this.isDataArrayTexture=!0,this.image={data:t,width:e,height:n,depth:s},this.magFilter=fn,this.minFilter=fn,this.wrapR=Ln,this.generateMipmaps=!1,this.flipY=!1,this.unpackAlignment=1,this.layerUpdates=new Set}addLayerUpdate(t){this.layerUpdates.add(t)}clearLayerUpdates(){this.layerUpdates.clear()}};var Ba=class extends nn{constructor(t=null,e=1,n=1,s=1){super(null),this.isData3DTexture=!0,this.image={data:t,width:e,height:n,depth:s},this.magFilter=fn,this.minFilter=fn,this.wrapR=Ln,this.generateMipmaps=!1,this.flipY=!1,this.unpackAlignment=1}};var qe=class{constructor(t=0,e=0,n=0,s=1){this.isQuaternion=!0,this._x=t,this._y=e,this._z=n,this._w=s}static slerpFlat(t,e,n,s,r,o,a){let c=n[s+0],l=n[s+1],h=n[s+2],u=n[s+3],p=r[o+0],f=r[o+1],g=r[o+2],y=r[o+3];if(a===0){t[e+0]=c,t[e+1]=l,t[e+2]=h,t[e+3]=u;return}if(a===1){t[e+0]=p,t[e+1]=f,t[e+2]=g,t[e+3]=y;return}if(u!==y||c!==p||l!==f||h!==g){let m=1-a,d=c*p+l*f+h*g+u*y,x=d>=0?1:-1,_=1-d*d;if(_>Number.EPSILON){let C=Math.sqrt(_),w=Math.atan2(C,d*x);m=Math.sin(m*w)/C,a=Math.sin(a*w)/C}let v=a*x;if(c=c*m+p*v,l=l*m+f*v,h=h*m+g*v,u=u*m+y*v,m===1-a){let C=1/Math.sqrt(c*c+l*l+h*h+u*u);c*=C,l*=C,h*=C,u*=C}}t[e]=c,t[e+1]=l,t[e+2]=h,t[e+3]=u}static multiplyQuaternionsFlat(t,e,n,s,r,o){let a=n[s],c=n[s+1],l=n[s+2],h=n[s+3],u=r[o],p=r[o+1],f=r[o+2],g=r[o+3];return t[e]=a*g+h*u+c*f-l*p,t[e+1]=c*g+h*p+l*u-a*f,t[e+2]=l*g+h*f+a*p-c*u,t[e+3]=h*g-a*u-c*p-l*f,t}get x(){return this._x}set x(t){this._x=t,this._onChangeCallback()}get y(){return this._y}set y(t){this._y=t,this._onChangeCallback()}get z(){return this._z}set z(t){this._z=t,this._onChangeCallback()}get w(){return this._w}set w(t){this._w=t,this._onChangeCallback()}set(t,e,n,s){return this._x=t,this._y=e,this._z=n,this._w=s,this._onChangeCallback(),this}clone(){return new this.constructor(this._x,this._y,this._z,this._w)}copy(t){return this._x=t.x,this._y=t.y,this._z=t.z,this._w=t.w,this._onChangeCallback(),this}setFromEuler(t,e=!0){let n=t._x,s=t._y,r=t._z,o=t._order,a=Math.cos,c=Math.sin,l=a(n/2),h=a(s/2),u=a(r/2),p=c(n/2),f=c(s/2),g=c(r/2);switch(o){case"XYZ":this._x=p*h*u+l*f*g,this._y=l*f*u-p*h*g,this._z=l*h*g+p*f*u,this._w=l*h*u-p*f*g;break;case"YXZ":this._x=p*h*u+l*f*g,this._y=l*f*u-p*h*g,this._z=l*h*g-p*f*u,this._w=l*h*u+p*f*g;break;case"ZXY":this._x=p*h*u-l*f*g,this._y=l*f*u+p*h*g,this._z=l*h*g+p*f*u,this._w=l*h*u-p*f*g;break;case"ZYX":this._x=p*h*u-l*f*g,this._y=l*f*u+p*h*g,this._z=l*h*g-p*f*u,this._w=l*h*u+p*f*g;break;case"YZX":this._x=p*h*u+l*f*g,this._y=l*f*u+p*h*g,this._z=l*h*g-p*f*u,this._w=l*h*u-p*f*g;break;case"XZY":this._x=p*h*u-l*f*g,this._y=l*f*u-p*h*g,this._z=l*h*g+p*f*u,this._w=l*h*u+p*f*g;break;default:console.warn("THREE.Quaternion: .setFromEuler() encountered an unknown order: "+o)}return e===!0&&this._onChangeCallback(),this}setFromAxisAngle(t,e){let n=e/2,s=Math.sin(n);return this._x=t.x*s,this._y=t.y*s,this._z=t.z*s,this._w=Math.cos(n),this._onChangeCallback(),this}setFromRotationMatrix(t){let e=t.elements,n=e[0],s=e[4],r=e[8],o=e[1],a=e[5],c=e[9],l=e[2],h=e[6],u=e[10],p=n+a+u;if(p>0){let f=.5/Math.sqrt(p+1);this._w=.25/f,this._x=(h-c)*f,this._y=(r-l)*f,this._z=(o-s)*f}else if(n>a&&n>u){let f=2*Math.sqrt(1+n-a-u);this._w=(h-c)/f,this._x=.25*f,this._y=(s+o)/f,this._z=(r+l)/f}else if(a>u){let f=2*Math.sqrt(1+a-n-u);this._w=(r-l)/f,this._x=(s+o)/f,this._y=.25*f,this._z=(c+h)/f}else{let f=2*Math.sqrt(1+u-n-a);this._w=(o-s)/f,this._x=(r+l)/f,this._y=(c+h)/f,this._z=.25*f}return this._onChangeCallback(),this}setFromUnitVectors(t,e){let n=t.dot(e)+1;return n<Number.EPSILON?(n=0,Math.abs(t.x)>Math.abs(t.z)?(this._x=-t.y,this._y=t.x,this._z=0,this._w=n):(this._x=0,this._y=-t.z,this._z=t.y,this._w=n)):(this._x=t.y*e.z-t.z*e.y,this._y=t.z*e.x-t.x*e.z,this._z=t.x*e.y-t.y*e.x,this._w=n),this.normalize()}angleTo(t){return 2*Math.acos(Math.abs(Ye(this.dot(t),-1,1)))}rotateTowards(t,e){let n=this.angleTo(t);if(n===0)return this;let s=Math.min(1,e/n);return this.slerp(t,s),this}identity(){return this.set(0,0,0,1)}invert(){return this.conjugate()}conjugate(){return this._x*=-1,this._y*=-1,this._z*=-1,this._onChangeCallback(),this}dot(t){return this._x*t._x+this._y*t._y+this._z*t._z+this._w*t._w}lengthSq(){return this._x*this._x+this._y*this._y+this._z*this._z+this._w*this._w}length(){return Math.sqrt(this._x*this._x+this._y*this._y+this._z*this._z+this._w*this._w)}normalize(){let t=this.length();return t===0?(this._x=0,this._y=0,this._z=0,this._w=1):(t=1/t,this._x=this._x*t,this._y=this._y*t,this._z=this._z*t,this._w=this._w*t),this._onChangeCallback(),this}multiply(t){return this.multiplyQuaternions(this,t)}premultiply(t){return this.multiplyQuaternions(t,this)}multiplyQuaternions(t,e){let n=t._x,s=t._y,r=t._z,o=t._w,a=e._x,c=e._y,l=e._z,h=e._w;return this._x=n*h+o*a+s*l-r*c,this._y=s*h+o*c+r*a-n*l,this._z=r*h+o*l+n*c-s*a,this._w=o*h-n*a-s*c-r*l,this._onChangeCallback(),this}slerp(t,e){if(e===0)return this;if(e===1)return this.copy(t);let n=this._x,s=this._y,r=this._z,o=this._w,a=o*t._w+n*t._x+s*t._y+r*t._z;if(a<0?(this._w=-t._w,this._x=-t._x,this._y=-t._y,this._z=-t._z,a=-a):this.copy(t),a>=1)return this._w=o,this._x=n,this._y=s,this._z=r,this;let c=1-a*a;if(c<=Number.EPSILON){let f=1-e;return this._w=f*o+e*this._w,this._x=f*n+e*this._x,this._y=f*s+e*this._y,this._z=f*r+e*this._z,this.normalize(),this}let l=Math.sqrt(c),h=Math.atan2(l,a),u=Math.sin((1-e)*h)/l,p=Math.sin(e*h)/l;return this._w=o*u+this._w*p,this._x=n*u+this._x*p,this._y=s*u+this._y*p,this._z=r*u+this._z*p,this._onChangeCallback(),this}slerpQuaternions(t,e,n){return this.copy(t).slerp(e,n)}random(){let t=2*Math.PI*Math.random(),e=2*Math.PI*Math.random(),n=Math.random(),s=Math.sqrt(1-n),r=Math.sqrt(n);return this.set(s*Math.sin(t),s*Math.cos(t),r*Math.sin(e),r*Math.cos(e))}equals(t){return t._x===this._x&&t._y===this._y&&t._z===this._z&&t._w===this._w}fromArray(t,e=0){return this._x=t[e],this._y=t[e+1],this._z=t[e+2],this._w=t[e+3],this._onChangeCallback(),this}toArray(t=[],e=0){return t[e]=this._x,t[e+1]=this._y,t[e+2]=this._z,t[e+3]=this._w,t}fromBufferAttribute(t,e){return this._x=t.getX(e),this._y=t.getY(e),this._z=t.getZ(e),this._w=t.getW(e),this._onChangeCallback(),this}toJSON(){return this.toArray()}_onChange(t){return this._onChangeCallback=t,this}_onChangeCallback(){}*[Symbol.iterator](){yield this._x,yield this._y,yield this._z,yield this._w}},U=class i{constructor(t=0,e=0,n=0){i.prototype.isVector3=!0,this.x=t,this.y=e,this.z=n}set(t,e,n){return n===void 0&&(n=this.z),this.x=t,this.y=e,this.z=n,this}setScalar(t){return this.x=t,this.y=t,this.z=t,this}setX(t){return this.x=t,this}setY(t){return this.y=t,this}setZ(t){return this.z=t,this}setComponent(t,e){switch(t){case 0:this.x=e;break;case 1:this.y=e;break;case 2:this.z=e;break;default:throw new Error("index is out of range: "+t)}return this}getComponent(t){switch(t){case 0:return this.x;case 1:return this.y;case 2:return this.z;default:throw new Error("index is out of range: "+t)}}clone(){return new this.constructor(this.x,this.y,this.z)}copy(t){return this.x=t.x,this.y=t.y,this.z=t.z,this}add(t){return this.x+=t.x,this.y+=t.y,this.z+=t.z,this}addScalar(t){return this.x+=t,this.y+=t,this.z+=t,this}addVectors(t,e){return this.x=t.x+e.x,this.y=t.y+e.y,this.z=t.z+e.z,this}addScaledVector(t,e){return this.x+=t.x*e,this.y+=t.y*e,this.z+=t.z*e,this}sub(t){return this.x-=t.x,this.y-=t.y,this.z-=t.z,this}subScalar(t){return this.x-=t,this.y-=t,this.z-=t,this}subVectors(t,e){return this.x=t.x-e.x,this.y=t.y-e.y,this.z=t.z-e.z,this}multiply(t){return this.x*=t.x,this.y*=t.y,this.z*=t.z,this}multiplyScalar(t){return this.x*=t,this.y*=t,this.z*=t,this}multiplyVectors(t,e){return this.x=t.x*e.x,this.y=t.y*e.y,this.z=t.z*e.z,this}applyEuler(t){return this.applyQuaternion(rl.setFromEuler(t))}applyAxisAngle(t,e){return this.applyQuaternion(rl.setFromAxisAngle(t,e))}applyMatrix3(t){let e=this.x,n=this.y,s=this.z,r=t.elements;return this.x=r[0]*e+r[3]*n+r[6]*s,this.y=r[1]*e+r[4]*n+r[7]*s,this.z=r[2]*e+r[5]*n+r[8]*s,this}applyNormalMatrix(t){return this.applyMatrix3(t).normalize()}applyMatrix4(t){let e=this.x,n=this.y,s=this.z,r=t.elements,o=1/(r[3]*e+r[7]*n+r[11]*s+r[15]);return this.x=(r[0]*e+r[4]*n+r[8]*s+r[12])*o,this.y=(r[1]*e+r[5]*n+r[9]*s+r[13])*o,this.z=(r[2]*e+r[6]*n+r[10]*s+r[14])*o,this}applyQuaternion(t){let e=this.x,n=this.y,s=this.z,r=t.x,o=t.y,a=t.z,c=t.w,l=2*(o*s-a*n),h=2*(a*e-r*s),u=2*(r*n-o*e);return this.x=e+c*l+o*u-a*h,this.y=n+c*h+a*l-r*u,this.z=s+c*u+r*h-o*l,this}project(t){return this.applyMatrix4(t.matrixWorldInverse).applyMatrix4(t.projectionMatrix)}unproject(t){return this.applyMatrix4(t.projectionMatrixInverse).applyMatrix4(t.matrixWorld)}transformDirection(t){let e=this.x,n=this.y,s=this.z,r=t.elements;return this.x=r[0]*e+r[4]*n+r[8]*s,this.y=r[1]*e+r[5]*n+r[9]*s,this.z=r[2]*e+r[6]*n+r[10]*s,this.normalize()}divide(t){return this.x/=t.x,this.y/=t.y,this.z/=t.z,this}divideScalar(t){return this.multiplyScalar(1/t)}min(t){return this.x=Math.min(this.x,t.x),this.y=Math.min(this.y,t.y),this.z=Math.min(this.z,t.z),this}max(t){return this.x=Math.max(this.x,t.x),this.y=Math.max(this.y,t.y),this.z=Math.max(this.z,t.z),this}clamp(t,e){return this.x=Math.max(t.x,Math.min(e.x,this.x)),this.y=Math.max(t.y,Math.min(e.y,this.y)),this.z=Math.max(t.z,Math.min(e.z,this.z)),this}clampScalar(t,e){return this.x=Math.max(t,Math.min(e,this.x)),this.y=Math.max(t,Math.min(e,this.y)),this.z=Math.max(t,Math.min(e,this.z)),this}clampLength(t,e){let n=this.length();return this.divideScalar(n||1).multiplyScalar(Math.max(t,Math.min(e,n)))}floor(){return this.x=Math.floor(this.x),this.y=Math.floor(this.y),this.z=Math.floor(this.z),this}ceil(){return this.x=Math.ceil(this.x),this.y=Math.ceil(this.y),this.z=Math.ceil(this.z),this}round(){return this.x=Math.round(this.x),this.y=Math.round(this.y),this.z=Math.round(this.z),this}roundToZero(){return this.x=Math.trunc(this.x),this.y=Math.trunc(this.y),this.z=Math.trunc(this.z),this}negate(){return this.x=-this.x,this.y=-this.y,this.z=-this.z,this}dot(t){return this.x*t.x+this.y*t.y+this.z*t.z}lengthSq(){return this.x*this.x+this.y*this.y+this.z*this.z}length(){return Math.sqrt(this.x*this.x+this.y*this.y+this.z*this.z)}manhattanLength(){return Math.abs(this.x)+Math.abs(this.y)+Math.abs(this.z)}normalize(){return this.divideScalar(this.length()||1)}setLength(t){return this.normalize().multiplyScalar(t)}lerp(t,e){return this.x+=(t.x-this.x)*e,this.y+=(t.y-this.y)*e,this.z+=(t.z-this.z)*e,this}lerpVectors(t,e,n){return this.x=t.x+(e.x-t.x)*n,this.y=t.y+(e.y-t.y)*n,this.z=t.z+(e.z-t.z)*n,this}cross(t){return this.crossVectors(this,t)}crossVectors(t,e){let n=t.x,s=t.y,r=t.z,o=e.x,a=e.y,c=e.z;return this.x=s*c-r*a,this.y=r*o-n*c,this.z=n*a-s*o,this}projectOnVector(t){let e=t.lengthSq();if(e===0)return this.set(0,0,0);let n=t.dot(this)/e;return this.copy(t).multiplyScalar(n)}projectOnPlane(t){return Eo.copy(this).projectOnVector(t),this.sub(Eo)}reflect(t){return this.sub(Eo.copy(t).multiplyScalar(2*this.dot(t)))}angleTo(t){let e=Math.sqrt(this.lengthSq()*t.lengthSq());if(e===0)return Math.PI/2;let n=this.dot(t)/e;return Math.acos(Ye(n,-1,1))}distanceTo(t){return Math.sqrt(this.distanceToSquared(t))}distanceToSquared(t){let e=this.x-t.x,n=this.y-t.y,s=this.z-t.z;return e*e+n*n+s*s}manhattanDistanceTo(t){return Math.abs(this.x-t.x)+Math.abs(this.y-t.y)+Math.abs(this.z-t.z)}setFromSpherical(t){return this.setFromSphericalCoords(t.radius,t.phi,t.theta)}setFromSphericalCoords(t,e,n){let s=Math.sin(e)*t;return this.x=s*Math.sin(n),this.y=Math.cos(e)*t,this.z=s*Math.cos(n),this}setFromCylindrical(t){return this.setFromCylindricalCoords(t.radius,t.theta,t.y)}setFromCylindricalCoords(t,e,n){return this.x=t*Math.sin(e),this.y=n,this.z=t*Math.cos(e),this}setFromMatrixPosition(t){let e=t.elements;return this.x=e[12],this.y=e[13],this.z=e[14],this}setFromMatrixScale(t){let e=this.setFromMatrixColumn(t,0).length(),n=this.setFromMatrixColumn(t,1).length(),s=this.setFromMatrixColumn(t,2).length();return this.x=e,this.y=n,this.z=s,this}setFromMatrixColumn(t,e){return this.fromArray(t.elements,e*4)}setFromMatrix3Column(t,e){return this.fromArray(t.elements,e*3)}setFromEuler(t){return this.x=t._x,this.y=t._y,this.z=t._z,this}setFromColor(t){return this.x=t.r,this.y=t.g,this.z=t.b,this}equals(t){return t.x===this.x&&t.y===this.y&&t.z===this.z}fromArray(t,e=0){return this.x=t[e],this.y=t[e+1],this.z=t[e+2],this}toArray(t=[],e=0){return t[e]=this.x,t[e+1]=this.y,t[e+2]=this.z,t}fromBufferAttribute(t,e){return this.x=t.getX(e),this.y=t.getY(e),this.z=t.getZ(e),this}random(){return this.x=Math.random(),this.y=Math.random(),this.z=Math.random(),this}randomDirection(){let t=Math.random()*Math.PI*2,e=Math.random()*2-1,n=Math.sqrt(1-e*e);return this.x=n*Math.cos(t),this.y=e,this.z=n*Math.sin(t),this}*[Symbol.iterator](){yield this.x,yield this.y,yield this.z}},Eo=new U,rl=new qe,Qn=class{constructor(t=new U(1/0,1/0,1/0),e=new U(-1/0,-1/0,-1/0)){this.isBox3=!0,this.min=t,this.max=e}set(t,e){return this.min.copy(t),this.max.copy(e),this}setFromArray(t){this.makeEmpty();for(let e=0,n=t.length;e<n;e+=3)this.expandByPoint(vn.fromArray(t,e));return this}setFromBufferAttribute(t){this.makeEmpty();for(let e=0,n=t.count;e<n;e++)this.expandByPoint(vn.fromBufferAttribute(t,e));return this}setFromPoints(t){this.makeEmpty();for(let e=0,n=t.length;e<n;e++)this.expandByPoint(t[e]);return this}setFromCenterAndSize(t,e){let n=vn.copy(e).multiplyScalar(.5);return this.min.copy(t).sub(n),this.max.copy(t).add(n),this}setFromObject(t,e=!1){return this.makeEmpty(),this.expandByObject(t,e)}clone(){return new this.constructor().copy(this)}copy(t){return this.min.copy(t.min),this.max.copy(t.max),this}makeEmpty(){return this.min.x=this.min.y=this.min.z=1/0,this.max.x=this.max.y=this.max.z=-1/0,this}isEmpty(){return this.max.x<this.min.x||this.max.y<this.min.y||this.max.z<this.min.z}getCenter(t){return this.isEmpty()?t.set(0,0,0):t.addVectors(this.min,this.max).multiplyScalar(.5)}getSize(t){return this.isEmpty()?t.set(0,0,0):t.subVectors(this.max,this.min)}expandByPoint(t){return this.min.min(t),this.max.max(t),this}expandByVector(t){return this.min.sub(t),this.max.add(t),this}expandByScalar(t){return this.min.addScalar(-t),this.max.addScalar(t),this}expandByObject(t,e=!1){t.updateWorldMatrix(!1,!1);let n=t.geometry;if(n!==void 0){let r=n.getAttribute("position");if(e===!0&&r!==void 0&&t.isInstancedMesh!==!0)for(let o=0,a=r.count;o<a;o++)t.isMesh===!0?t.getVertexPosition(o,vn):vn.fromBufferAttribute(r,o),vn.applyMatrix4(t.matrixWorld),this.expandByPoint(vn);else t.boundingBox!==void 0?(t.boundingBox===null&&t.computeBoundingBox(),Qs.copy(t.boundingBox)):(n.boundingBox===null&&n.computeBoundingBox(),Qs.copy(n.boundingBox)),Qs.applyMatrix4(t.matrixWorld),this.union(Qs)}let s=t.children;for(let r=0,o=s.length;r<o;r++)this.expandByObject(s[r],e);return this}containsPoint(t){return t.x>=this.min.x&&t.x<=this.max.x&&t.y>=this.min.y&&t.y<=this.max.y&&t.z>=this.min.z&&t.z<=this.max.z}containsBox(t){return this.min.x<=t.min.x&&t.max.x<=this.max.x&&this.min.y<=t.min.y&&t.max.y<=this.max.y&&this.min.z<=t.min.z&&t.max.z<=this.max.z}getParameter(t,e){return e.set((t.x-this.min.x)/(this.max.x-this.min.x),(t.y-this.min.y)/(this.max.y-this.min.y),(t.z-this.min.z)/(this.max.z-this.min.z))}intersectsBox(t){return t.max.x>=this.min.x&&t.min.x<=this.max.x&&t.max.y>=this.min.y&&t.min.y<=this.max.y&&t.max.z>=this.min.z&&t.min.z<=this.max.z}intersectsSphere(t){return this.clampPoint(t.center,vn),vn.distanceToSquared(t.center)<=t.radius*t.radius}intersectsPlane(t){let e,n;return t.normal.x>0?(e=t.normal.x*this.min.x,n=t.normal.x*this.max.x):(e=t.normal.x*this.max.x,n=t.normal.x*this.min.x),t.normal.y>0?(e+=t.normal.y*this.min.y,n+=t.normal.y*this.max.y):(e+=t.normal.y*this.max.y,n+=t.normal.y*this.min.y),t.normal.z>0?(e+=t.normal.z*this.min.z,n+=t.normal.z*this.max.z):(e+=t.normal.z*this.max.z,n+=t.normal.z*this.min.z),e<=-t.constant&&n>=-t.constant}intersectsTriangle(t){if(this.isEmpty())return!1;this.getCenter(Es),tr.subVectors(this.max,Es),Oi.subVectors(t.a,Es),Bi.subVectors(t.b,Es),zi.subVectors(t.c,Es),si.subVectors(Bi,Oi),ri.subVectors(zi,Bi),yi.subVectors(Oi,zi);let e=[0,-si.z,si.y,0,-ri.z,ri.y,0,-yi.z,yi.y,si.z,0,-si.x,ri.z,0,-ri.x,yi.z,0,-yi.x,-si.y,si.x,0,-ri.y,ri.x,0,-yi.y,yi.x,0];return!To(e,Oi,Bi,zi,tr)||(e=[1,0,0,0,1,0,0,0,1],!To(e,Oi,Bi,zi,tr))?!1:(er.crossVectors(si,ri),e=[er.x,er.y,er.z],To(e,Oi,Bi,zi,tr))}clampPoint(t,e){return e.copy(t).clamp(this.min,this.max)}distanceToPoint(t){return this.clampPoint(t,vn).distanceTo(t)}getBoundingSphere(t){return this.isEmpty()?t.makeEmpty():(this.getCenter(t.center),t.radius=this.getSize(vn).length()*.5),t}intersect(t){return this.min.max(t.min),this.max.min(t.max),this.isEmpty()&&this.makeEmpty(),this}union(t){return this.min.min(t.min),this.max.max(t.max),this}applyMatrix4(t){return this.isEmpty()?this:(Hn[0].set(this.min.x,this.min.y,this.min.z).applyMatrix4(t),Hn[1].set(this.min.x,this.min.y,this.max.z).applyMatrix4(t),Hn[2].set(this.min.x,this.max.y,this.min.z).applyMatrix4(t),Hn[3].set(this.min.x,this.max.y,this.max.z).applyMatrix4(t),Hn[4].set(this.max.x,this.min.y,this.min.z).applyMatrix4(t),Hn[5].set(this.max.x,this.min.y,this.max.z).applyMatrix4(t),Hn[6].set(this.max.x,this.max.y,this.min.z).applyMatrix4(t),Hn[7].set(this.max.x,this.max.y,this.max.z).applyMatrix4(t),this.setFromPoints(Hn),this)}translate(t){return this.min.add(t),this.max.add(t),this}equals(t){return t.min.equals(this.min)&&t.max.equals(this.max)}},Hn=[new U,new U,new U,new U,new U,new U,new U,new U],vn=new U,Qs=new Qn,Oi=new U,Bi=new U,zi=new U,si=new U,ri=new U,yi=new U,Es=new U,tr=new U,er=new U,vi=new U;function To(i,t,e,n,s){for(let r=0,o=i.length-3;r<=o;r+=3){vi.fromArray(i,r);let a=s.x*Math.abs(vi.x)+s.y*Math.abs(vi.y)+s.z*Math.abs(vi.z),c=t.dot(vi),l=e.dot(vi),h=n.dot(vi);if(Math.max(-Math.max(c,l,h),Math.min(c,l,h))>a)return!1}return!0}var bd=new Qn,Ts=new U,wo=new U,mi=class{constructor(t=new U,e=-1){this.isSphere=!0,this.center=t,this.radius=e}set(t,e){return this.center.copy(t),this.radius=e,this}setFromPoints(t,e){let n=this.center;e!==void 0?n.copy(e):bd.setFromPoints(t).getCenter(n);let s=0;for(let r=0,o=t.length;r<o;r++)s=Math.max(s,n.distanceToSquared(t[r]));return this.radius=Math.sqrt(s),this}copy(t){return this.center.copy(t.center),this.radius=t.radius,this}isEmpty(){return this.radius<0}makeEmpty(){return this.center.set(0,0,0),this.radius=-1,this}containsPoint(t){return t.distanceToSquared(this.center)<=this.radius*this.radius}distanceToPoint(t){return t.distanceTo(this.center)-this.radius}intersectsSphere(t){let e=this.radius+t.radius;return t.center.distanceToSquared(this.center)<=e*e}intersectsBox(t){return t.intersectsSphere(this)}intersectsPlane(t){return Math.abs(t.distanceToPoint(this.center))<=this.radius}clampPoint(t,e){let n=this.center.distanceToSquared(t);return e.copy(t),n>this.radius*this.radius&&(e.sub(this.center).normalize(),e.multiplyScalar(this.radius).add(this.center)),e}getBoundingBox(t){return this.isEmpty()?(t.makeEmpty(),t):(t.set(this.center,this.center),t.expandByScalar(this.radius),t)}applyMatrix4(t){return this.center.applyMatrix4(t),this.radius=this.radius*t.getMaxScaleOnAxis(),this}translate(t){return this.center.add(t),this}expandByPoint(t){if(this.isEmpty())return this.center.copy(t),this.radius=0,this;Ts.subVectors(t,this.center);let e=Ts.lengthSq();if(e>this.radius*this.radius){let n=Math.sqrt(e),s=(n-this.radius)*.5;this.center.addScaledVector(Ts,s/n),this.radius+=s}return this}union(t){return t.isEmpty()?this:this.isEmpty()?(this.copy(t),this):(this.center.equals(t.center)===!0?this.radius=Math.max(this.radius,t.radius):(wo.subVectors(t.center,this.center).setLength(t.radius),this.expandByPoint(Ts.copy(t.center).add(wo)),this.expandByPoint(Ts.copy(t.center).sub(wo))),this)}equals(t){return t.center.equals(this.center)&&t.radius===this.radius}clone(){return new this.constructor().copy(this)}},Vn=new U,Ao=new U,nr=new U,oi=new U,Co=new U,ir=new U,Ro=new U,hs=class{constructor(t=new U,e=new U(0,0,-1)){this.origin=t,this.direction=e}set(t,e){return this.origin.copy(t),this.direction.copy(e),this}copy(t){return this.origin.copy(t.origin),this.direction.copy(t.direction),this}at(t,e){return e.copy(this.origin).addScaledVector(this.direction,t)}lookAt(t){return this.direction.copy(t).sub(this.origin).normalize(),this}recast(t){return this.origin.copy(this.at(t,Vn)),this}closestPointToPoint(t,e){e.subVectors(t,this.origin);let n=e.dot(this.direction);return n<0?e.copy(this.origin):e.copy(this.origin).addScaledVector(this.direction,n)}distanceToPoint(t){return Math.sqrt(this.distanceSqToPoint(t))}distanceSqToPoint(t){let e=Vn.subVectors(t,this.origin).dot(this.direction);return e<0?this.origin.distanceToSquared(t):(Vn.copy(this.origin).addScaledVector(this.direction,e),Vn.distanceToSquared(t))}distanceSqToSegment(t,e,n,s){Ao.copy(t).add(e).multiplyScalar(.5),nr.copy(e).sub(t).normalize(),oi.copy(this.origin).sub(Ao);let r=t.distanceTo(e)*.5,o=-this.direction.dot(nr),a=oi.dot(this.direction),c=-oi.dot(nr),l=oi.lengthSq(),h=Math.abs(1-o*o),u,p,f,g;if(h>0)if(u=o*c-a,p=o*a-c,g=r*h,u>=0)if(p>=-g)if(p<=g){let y=1/h;u*=y,p*=y,f=u*(u+o*p+2*a)+p*(o*u+p+2*c)+l}else p=r,u=Math.max(0,-(o*p+a)),f=-u*u+p*(p+2*c)+l;else p=-r,u=Math.max(0,-(o*p+a)),f=-u*u+p*(p+2*c)+l;else p<=-g?(u=Math.max(0,-(-o*r+a)),p=u>0?-r:Math.min(Math.max(-r,-c),r),f=-u*u+p*(p+2*c)+l):p<=g?(u=0,p=Math.min(Math.max(-r,-c),r),f=p*(p+2*c)+l):(u=Math.max(0,-(o*r+a)),p=u>0?r:Math.min(Math.max(-r,-c),r),f=-u*u+p*(p+2*c)+l);else p=o>0?-r:r,u=Math.max(0,-(o*p+a)),f=-u*u+p*(p+2*c)+l;return n&&n.copy(this.origin).addScaledVector(this.direction,u),s&&s.copy(Ao).addScaledVector(nr,p),f}intersectSphere(t,e){Vn.subVectors(t.center,this.origin);let n=Vn.dot(this.direction),s=Vn.dot(Vn)-n*n,r=t.radius*t.radius;if(s>r)return null;let o=Math.sqrt(r-s),a=n-o,c=n+o;return c<0?null:a<0?this.at(c,e):this.at(a,e)}intersectsSphere(t){return this.distanceSqToPoint(t.center)<=t.radius*t.radius}distanceToPlane(t){let e=t.normal.dot(this.direction);if(e===0)return t.distanceToPoint(this.origin)===0?0:null;let n=-(this.origin.dot(t.normal)+t.constant)/e;return n>=0?n:null}intersectPlane(t,e){let n=this.distanceToPlane(t);return n===null?null:this.at(n,e)}intersectsPlane(t){let e=t.distanceToPoint(this.origin);return e===0||t.normal.dot(this.direction)*e<0}intersectBox(t,e){let n,s,r,o,a,c,l=1/this.direction.x,h=1/this.direction.y,u=1/this.direction.z,p=this.origin;return l>=0?(n=(t.min.x-p.x)*l,s=(t.max.x-p.x)*l):(n=(t.max.x-p.x)*l,s=(t.min.x-p.x)*l),h>=0?(r=(t.min.y-p.y)*h,o=(t.max.y-p.y)*h):(r=(t.max.y-p.y)*h,o=(t.min.y-p.y)*h),n>o||r>s||((r>n||isNaN(n))&&(n=r),(o<s||isNaN(s))&&(s=o),u>=0?(a=(t.min.z-p.z)*u,c=(t.max.z-p.z)*u):(a=(t.max.z-p.z)*u,c=(t.min.z-p.z)*u),n>c||a>s)||((a>n||n!==n)&&(n=a),(c<s||s!==s)&&(s=c),s<0)?null:this.at(n>=0?n:s,e)}intersectsBox(t){return this.intersectBox(t,Vn)!==null}intersectTriangle(t,e,n,s,r){Co.subVectors(e,t),ir.subVectors(n,t),Ro.crossVectors(Co,ir);let o=this.direction.dot(Ro),a;if(o>0){if(s)return null;a=1}else if(o<0)a=-1,o=-o;else return null;oi.subVectors(this.origin,t);let c=a*this.direction.dot(ir.crossVectors(oi,ir));if(c<0)return null;let l=a*this.direction.dot(Co.cross(oi));if(l<0||c+l>o)return null;let h=-a*oi.dot(Ro);return h<0?null:this.at(h/o,r)}applyMatrix4(t){return this.origin.applyMatrix4(t),this.direction.transformDirection(t),this}equals(t){return t.origin.equals(this.origin)&&t.direction.equals(this.direction)}clone(){return new this.constructor().copy(this)}},ce=class i{constructor(t,e,n,s,r,o,a,c,l,h,u,p,f,g,y,m){i.prototype.isMatrix4=!0,this.elements=[1,0,0,0,0,1,0,0,0,0,1,0,0,0,0,1],t!==void 0&&this.set(t,e,n,s,r,o,a,c,l,h,u,p,f,g,y,m)}set(t,e,n,s,r,o,a,c,l,h,u,p,f,g,y,m){let d=this.elements;return d[0]=t,d[4]=e,d[8]=n,d[12]=s,d[1]=r,d[5]=o,d[9]=a,d[13]=c,d[2]=l,d[6]=h,d[10]=u,d[14]=p,d[3]=f,d[7]=g,d[11]=y,d[15]=m,this}identity(){return this.set(1,0,0,0,0,1,0,0,0,0,1,0,0,0,0,1),this}clone(){return new i().fromArray(this.elements)}copy(t){let e=this.elements,n=t.elements;return e[0]=n[0],e[1]=n[1],e[2]=n[2],e[3]=n[3],e[4]=n[4],e[5]=n[5],e[6]=n[6],e[7]=n[7],e[8]=n[8],e[9]=n[9],e[10]=n[10],e[11]=n[11],e[12]=n[12],e[13]=n[13],e[14]=n[14],e[15]=n[15],this}copyPosition(t){let e=this.elements,n=t.elements;return e[12]=n[12],e[13]=n[13],e[14]=n[14],this}setFromMatrix3(t){let e=t.elements;return this.set(e[0],e[3],e[6],0,e[1],e[4],e[7],0,e[2],e[5],e[8],0,0,0,0,1),this}extractBasis(t,e,n){return t.setFromMatrixColumn(this,0),e.setFromMatrixColumn(this,1),n.setFromMatrixColumn(this,2),this}makeBasis(t,e,n){return this.set(t.x,e.x,n.x,0,t.y,e.y,n.y,0,t.z,e.z,n.z,0,0,0,0,1),this}extractRotation(t){let e=this.elements,n=t.elements,s=1/ki.setFromMatrixColumn(t,0).length(),r=1/ki.setFromMatrixColumn(t,1).length(),o=1/ki.setFromMatrixColumn(t,2).length();return e[0]=n[0]*s,e[1]=n[1]*s,e[2]=n[2]*s,e[3]=0,e[4]=n[4]*r,e[5]=n[5]*r,e[6]=n[6]*r,e[7]=0,e[8]=n[8]*o,e[9]=n[9]*o,e[10]=n[10]*o,e[11]=0,e[12]=0,e[13]=0,e[14]=0,e[15]=1,this}makeRotationFromEuler(t){let e=this.elements,n=t.x,s=t.y,r=t.z,o=Math.cos(n),a=Math.sin(n),c=Math.cos(s),l=Math.sin(s),h=Math.cos(r),u=Math.sin(r);if(t.order==="XYZ"){let p=o*h,f=o*u,g=a*h,y=a*u;e[0]=c*h,e[4]=-c*u,e[8]=l,e[1]=f+g*l,e[5]=p-y*l,e[9]=-a*c,e[2]=y-p*l,e[6]=g+f*l,e[10]=o*c}else if(t.order==="YXZ"){let p=c*h,f=c*u,g=l*h,y=l*u;e[0]=p+y*a,e[4]=g*a-f,e[8]=o*l,e[1]=o*u,e[5]=o*h,e[9]=-a,e[2]=f*a-g,e[6]=y+p*a,e[10]=o*c}else if(t.order==="ZXY"){let p=c*h,f=c*u,g=l*h,y=l*u;e[0]=p-y*a,e[4]=-o*u,e[8]=g+f*a,e[1]=f+g*a,e[5]=o*h,e[9]=y-p*a,e[2]=-o*l,e[6]=a,e[10]=o*c}else if(t.order==="ZYX"){let p=o*h,f=o*u,g=a*h,y=a*u;e[0]=c*h,e[4]=g*l-f,e[8]=p*l+y,e[1]=c*u,e[5]=y*l+p,e[9]=f*l-g,e[2]=-l,e[6]=a*c,e[10]=o*c}else if(t.order==="YZX"){let p=o*c,f=o*l,g=a*c,y=a*l;e[0]=c*h,e[4]=y-p*u,e[8]=g*u+f,e[1]=u,e[5]=o*h,e[9]=-a*h,e[2]=-l*h,e[6]=f*u+g,e[10]=p-y*u}else if(t.order==="XZY"){let p=o*c,f=o*l,g=a*c,y=a*l;e[0]=c*h,e[4]=-u,e[8]=l*h,e[1]=p*u+y,e[5]=o*h,e[9]=f*u-g,e[2]=g*u-f,e[6]=a*h,e[10]=y*u+p}return e[3]=0,e[7]=0,e[11]=0,e[12]=0,e[13]=0,e[14]=0,e[15]=1,this}makeRotationFromQuaternion(t){return this.compose(Ed,t,Td)}lookAt(t,e,n){let s=this.elements;return ln.subVectors(t,e),ln.lengthSq()===0&&(ln.z=1),ln.normalize(),ai.crossVectors(n,ln),ai.lengthSq()===0&&(Math.abs(n.z)===1?ln.x+=1e-4:ln.z+=1e-4,ln.normalize(),ai.crossVectors(n,ln)),ai.normalize(),sr.crossVectors(ln,ai),s[0]=ai.x,s[4]=sr.x,s[8]=ln.x,s[1]=ai.y,s[5]=sr.y,s[9]=ln.y,s[2]=ai.z,s[6]=sr.z,s[10]=ln.z,this}multiply(t){return this.multiplyMatrices(this,t)}premultiply(t){return this.multiplyMatrices(t,this)}multiplyMatrices(t,e){let n=t.elements,s=e.elements,r=this.elements,o=n[0],a=n[4],c=n[8],l=n[12],h=n[1],u=n[5],p=n[9],f=n[13],g=n[2],y=n[6],m=n[10],d=n[14],x=n[3],_=n[7],v=n[11],C=n[15],w=s[0],A=s[4],P=s[8],E=s[12],M=s[1],R=s[5],F=s[9],V=s[13],z=s[2],k=s[6],I=s[10],Y=s[14],B=s[3],Q=s[7],ut=s[11],Et=s[15];return r[0]=o*w+a*M+c*z+l*B,r[4]=o*A+a*R+c*k+l*Q,r[8]=o*P+a*F+c*I+l*ut,r[12]=o*E+a*V+c*Y+l*Et,r[1]=h*w+u*M+p*z+f*B,r[5]=h*A+u*R+p*k+f*Q,r[9]=h*P+u*F+p*I+f*ut,r[13]=h*E+u*V+p*Y+f*Et,r[2]=g*w+y*M+m*z+d*B,r[6]=g*A+y*R+m*k+d*Q,r[10]=g*P+y*F+m*I+d*ut,r[14]=g*E+y*V+m*Y+d*Et,r[3]=x*w+_*M+v*z+C*B,r[7]=x*A+_*R+v*k+C*Q,r[11]=x*P+_*F+v*I+C*ut,r[15]=x*E+_*V+v*Y+C*Et,this}multiplyScalar(t){let e=this.elements;return e[0]*=t,e[4]*=t,e[8]*=t,e[12]*=t,e[1]*=t,e[5]*=t,e[9]*=t,e[13]*=t,e[2]*=t,e[6]*=t,e[10]*=t,e[14]*=t,e[3]*=t,e[7]*=t,e[11]*=t,e[15]*=t,this}determinant(){let t=this.elements,e=t[0],n=t[4],s=t[8],r=t[12],o=t[1],a=t[5],c=t[9],l=t[13],h=t[2],u=t[6],p=t[10],f=t[14],g=t[3],y=t[7],m=t[11],d=t[15];return g*(+r*c*u-s*l*u-r*a*p+n*l*p+s*a*f-n*c*f)+y*(+e*c*f-e*l*p+r*o*p-s*o*f+s*l*h-r*c*h)+m*(+e*l*u-e*a*f-r*o*u+n*o*f+r*a*h-n*l*h)+d*(-s*a*h-e*c*u+e*a*p+s*o*u-n*o*p+n*c*h)}transpose(){let t=this.elements,e;return e=t[1],t[1]=t[4],t[4]=e,e=t[2],t[2]=t[8],t[8]=e,e=t[6],t[6]=t[9],t[9]=e,e=t[3],t[3]=t[12],t[12]=e,e=t[7],t[7]=t[13],t[13]=e,e=t[11],t[11]=t[14],t[14]=e,this}setPosition(t,e,n){let s=this.elements;return t.isVector3?(s[12]=t.x,s[13]=t.y,s[14]=t.z):(s[12]=t,s[13]=e,s[14]=n),this}invert(){let t=this.elements,e=t[0],n=t[1],s=t[2],r=t[3],o=t[4],a=t[5],c=t[6],l=t[7],h=t[8],u=t[9],p=t[10],f=t[11],g=t[12],y=t[13],m=t[14],d=t[15],x=u*m*l-y*p*l+y*c*f-a*m*f-u*c*d+a*p*d,_=g*p*l-h*m*l-g*c*f+o*m*f+h*c*d-o*p*d,v=h*y*l-g*u*l+g*a*f-o*y*f-h*a*d+o*u*d,C=g*u*c-h*y*c-g*a*p+o*y*p+h*a*m-o*u*m,w=e*x+n*_+s*v+r*C;if(w===0)return this.set(0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0);let A=1/w;return t[0]=x*A,t[1]=(y*p*r-u*m*r-y*s*f+n*m*f+u*s*d-n*p*d)*A,t[2]=(a*m*r-y*c*r+y*s*l-n*m*l-a*s*d+n*c*d)*A,t[3]=(u*c*r-a*p*r-u*s*l+n*p*l+a*s*f-n*c*f)*A,t[4]=_*A,t[5]=(h*m*r-g*p*r+g*s*f-e*m*f-h*s*d+e*p*d)*A,t[6]=(g*c*r-o*m*r-g*s*l+e*m*l+o*s*d-e*c*d)*A,t[7]=(o*p*r-h*c*r+h*s*l-e*p*l-o*s*f+e*c*f)*A,t[8]=v*A,t[9]=(g*u*r-h*y*r-g*n*f+e*y*f+h*n*d-e*u*d)*A,t[10]=(o*y*r-g*a*r+g*n*l-e*y*l-o*n*d+e*a*d)*A,t[11]=(h*a*r-o*u*r-h*n*l+e*u*l+o*n*f-e*a*f)*A,t[12]=C*A,t[13]=(h*y*s-g*u*s+g*n*p-e*y*p-h*n*m+e*u*m)*A,t[14]=(g*a*s-o*y*s-g*n*c+e*y*c+o*n*m-e*a*m)*A,t[15]=(o*u*s-h*a*s+h*n*c-e*u*c-o*n*p+e*a*p)*A,this}scale(t){let e=this.elements,n=t.x,s=t.y,r=t.z;return e[0]*=n,e[4]*=s,e[8]*=r,e[1]*=n,e[5]*=s,e[9]*=r,e[2]*=n,e[6]*=s,e[10]*=r,e[3]*=n,e[7]*=s,e[11]*=r,this}getMaxScaleOnAxis(){let t=this.elements,e=t[0]*t[0]+t[1]*t[1]+t[2]*t[2],n=t[4]*t[4]+t[5]*t[5]+t[6]*t[6],s=t[8]*t[8]+t[9]*t[9]+t[10]*t[10];return Math.sqrt(Math.max(e,n,s))}makeTranslation(t,e,n){return t.isVector3?this.set(1,0,0,t.x,0,1,0,t.y,0,0,1,t.z,0,0,0,1):this.set(1,0,0,t,0,1,0,e,0,0,1,n,0,0,0,1),this}makeRotationX(t){let e=Math.cos(t),n=Math.sin(t);return this.set(1,0,0,0,0,e,-n,0,0,n,e,0,0,0,0,1),this}makeRotationY(t){let e=Math.cos(t),n=Math.sin(t);return this.set(e,0,n,0,0,1,0,0,-n,0,e,0,0,0,0,1),this}makeRotationZ(t){let e=Math.cos(t),n=Math.sin(t);return this.set(e,-n,0,0,n,e,0,0,0,0,1,0,0,0,0,1),this}makeRotationAxis(t,e){let n=Math.cos(e),s=Math.sin(e),r=1-n,o=t.x,a=t.y,c=t.z,l=r*o,h=r*a;return this.set(l*o+n,l*a-s*c,l*c+s*a,0,l*a+s*c,h*a+n,h*c-s*o,0,l*c-s*a,h*c+s*o,r*c*c+n,0,0,0,0,1),this}makeScale(t,e,n){return this.set(t,0,0,0,0,e,0,0,0,0,n,0,0,0,0,1),this}makeShear(t,e,n,s,r,o){return this.set(1,n,r,0,t,1,o,0,e,s,1,0,0,0,0,1),this}compose(t,e,n){let s=this.elements,r=e._x,o=e._y,a=e._z,c=e._w,l=r+r,h=o+o,u=a+a,p=r*l,f=r*h,g=r*u,y=o*h,m=o*u,d=a*u,x=c*l,_=c*h,v=c*u,C=n.x,w=n.y,A=n.z;return s[0]=(1-(y+d))*C,s[1]=(f+v)*C,s[2]=(g-_)*C,s[3]=0,s[4]=(f-v)*w,s[5]=(1-(p+d))*w,s[6]=(m+x)*w,s[7]=0,s[8]=(g+_)*A,s[9]=(m-x)*A,s[10]=(1-(p+y))*A,s[11]=0,s[12]=t.x,s[13]=t.y,s[14]=t.z,s[15]=1,this}decompose(t,e,n){let s=this.elements,r=ki.set(s[0],s[1],s[2]).length(),o=ki.set(s[4],s[5],s[6]).length(),a=ki.set(s[8],s[9],s[10]).length();this.determinant()<0&&(r=-r),t.x=s[12],t.y=s[13],t.z=s[14],Mn.copy(this);let l=1/r,h=1/o,u=1/a;return Mn.elements[0]*=l,Mn.elements[1]*=l,Mn.elements[2]*=l,Mn.elements[4]*=h,Mn.elements[5]*=h,Mn.elements[6]*=h,Mn.elements[8]*=u,Mn.elements[9]*=u,Mn.elements[10]*=u,e.setFromRotationMatrix(Mn),n.x=r,n.y=o,n.z=a,this}makePerspective(t,e,n,s,r,o,a=$n){let c=this.elements,l=2*r/(e-t),h=2*r/(n-s),u=(e+t)/(e-t),p=(n+s)/(n-s),f,g;if(a===$n)f=-(o+r)/(o-r),g=-2*o*r/(o-r);else if(a===Lr)f=-o/(o-r),g=-o*r/(o-r);else throw new Error("THREE.Matrix4.makePerspective(): Invalid coordinate system: "+a);return c[0]=l,c[4]=0,c[8]=u,c[12]=0,c[1]=0,c[5]=h,c[9]=p,c[13]=0,c[2]=0,c[6]=0,c[10]=f,c[14]=g,c[3]=0,c[7]=0,c[11]=-1,c[15]=0,this}makeOrthographic(t,e,n,s,r,o,a=$n){let c=this.elements,l=1/(e-t),h=1/(n-s),u=1/(o-r),p=(e+t)*l,f=(n+s)*h,g,y;if(a===$n)g=(o+r)*u,y=-2*u;else if(a===Lr)g=r*u,y=-1*u;else throw new Error("THREE.Matrix4.makeOrthographic(): Invalid coordinate system: "+a);return c[0]=2*l,c[4]=0,c[8]=0,c[12]=-p,c[1]=0,c[5]=2*h,c[9]=0,c[13]=-f,c[2]=0,c[6]=0,c[10]=y,c[14]=-g,c[3]=0,c[7]=0,c[11]=0,c[15]=1,this}equals(t){let e=this.elements,n=t.elements;for(let s=0;s<16;s++)if(e[s]!==n[s])return!1;return!0}fromArray(t,e=0){for(let n=0;n<16;n++)this.elements[n]=t[n+e];return this}toArray(t=[],e=0){let n=this.elements;return t[e]=n[0],t[e+1]=n[1],t[e+2]=n[2],t[e+3]=n[3],t[e+4]=n[4],t[e+5]=n[5],t[e+6]=n[6],t[e+7]=n[7],t[e+8]=n[8],t[e+9]=n[9],t[e+10]=n[10],t[e+11]=n[11],t[e+12]=n[12],t[e+13]=n[13],t[e+14]=n[14],t[e+15]=n[15],t}},ki=new U,Mn=new ce,Ed=new U(0,0,0),Td=new U(1,1,1),ai=new U,sr=new U,ln=new U,ol=new ce,al=new qe,sn=class i{constructor(t=0,e=0,n=0,s=i.DEFAULT_ORDER){this.isEuler=!0,this._x=t,this._y=e,this._z=n,this._order=s}get x(){return this._x}set x(t){this._x=t,this._onChangeCallback()}get y(){return this._y}set y(t){this._y=t,this._onChangeCallback()}get z(){return this._z}set z(t){this._z=t,this._onChangeCallback()}get order(){return this._order}set order(t){this._order=t,this._onChangeCallback()}set(t,e,n,s=this._order){return this._x=t,this._y=e,this._z=n,this._order=s,this._onChangeCallback(),this}clone(){return new this.constructor(this._x,this._y,this._z,this._order)}copy(t){return this._x=t._x,this._y=t._y,this._z=t._z,this._order=t._order,this._onChangeCallback(),this}setFromRotationMatrix(t,e=this._order,n=!0){let s=t.elements,r=s[0],o=s[4],a=s[8],c=s[1],l=s[5],h=s[9],u=s[2],p=s[6],f=s[10];switch(e){case"XYZ":this._y=Math.asin(Ye(a,-1,1)),Math.abs(a)<.9999999?(this._x=Math.atan2(-h,f),this._z=Math.atan2(-o,r)):(this._x=Math.atan2(p,l),this._z=0);break;case"YXZ":this._x=Math.asin(-Ye(h,-1,1)),Math.abs(h)<.9999999?(this._y=Math.atan2(a,f),this._z=Math.atan2(c,l)):(this._y=Math.atan2(-u,r),this._z=0);break;case"ZXY":this._x=Math.asin(Ye(p,-1,1)),Math.abs(p)<.9999999?(this._y=Math.atan2(-u,f),this._z=Math.atan2(-o,l)):(this._y=0,this._z=Math.atan2(c,r));break;case"ZYX":this._y=Math.asin(-Ye(u,-1,1)),Math.abs(u)<.9999999?(this._x=Math.atan2(p,f),this._z=Math.atan2(c,r)):(this._x=0,this._z=Math.atan2(-o,l));break;case"YZX":this._z=Math.asin(Ye(c,-1,1)),Math.abs(c)<.9999999?(this._x=Math.atan2(-h,l),this._y=Math.atan2(-u,r)):(this._x=0,this._y=Math.atan2(a,f));break;case"XZY":this._z=Math.asin(-Ye(o,-1,1)),Math.abs(o)<.9999999?(this._x=Math.atan2(p,l),this._y=Math.atan2(a,r)):(this._x=Math.atan2(-h,f),this._y=0);break;default:console.warn("THREE.Euler: .setFromRotationMatrix() encountered an unknown order: "+e)}return this._order=e,n===!0&&this._onChangeCallback(),this}setFromQuaternion(t,e,n){return ol.makeRotationFromQuaternion(t),this.setFromRotationMatrix(ol,e,n)}setFromVector3(t,e=this._order){return this.set(t.x,t.y,t.z,e)}reorder(t){return al.setFromEuler(this),this.setFromQuaternion(al,t)}equals(t){return t._x===this._x&&t._y===this._y&&t._z===this._z&&t._order===this._order}fromArray(t){return this._x=t[0],this._y=t[1],this._z=t[2],t[3]!==void 0&&(this._order=t[3]),this._onChangeCallback(),this}toArray(t=[],e=0){return t[e]=this._x,t[e+1]=this._y,t[e+2]=this._z,t[e+3]=this._order,t}_onChange(t){return this._onChangeCallback=t,this}_onChangeCallback(){}*[Symbol.iterator](){yield this._x,yield this._y,yield this._z,yield this._order}};sn.DEFAULT_ORDER="XYZ";var Fr=class{constructor(){this.mask=1}set(t){this.mask=(1<<t|0)>>>0}enable(t){this.mask|=1<<t|0}enableAll(){this.mask=-1}toggle(t){this.mask^=1<<t|0}disable(t){this.mask&=~(1<<t|0)}disableAll(){this.mask=0}test(t){return(this.mask&t.mask)!==0}isEnabled(t){return(this.mask&(1<<t|0))!==0}},wd=0,cl=new U,Hi=new qe,Gn=new ce,rr=new U,ws=new U,Ad=new U,Cd=new qe,ll=new U(1,0,0),hl=new U(0,1,0),ul=new U(0,0,1),dl={type:"added"},Rd={type:"removed"},Vi={type:"childadded",child:null},Po={type:"childremoved",child:null},Re=class i extends jn{constructor(){super(),this.isObject3D=!0,Object.defineProperty(this,"id",{value:wd++}),this.uuid=Zn(),this.name="",this.type="Object3D",this.parent=null,this.children=[],this.up=i.DEFAULT_UP.clone();let t=new U,e=new sn,n=new qe,s=new U(1,1,1);function r(){n.setFromEuler(e,!1)}function o(){e.setFromQuaternion(n,void 0,!1)}e._onChange(r),n._onChange(o),Object.defineProperties(this,{position:{configurable:!0,enumerable:!0,value:t},rotation:{configurable:!0,enumerable:!0,value:e},quaternion:{configurable:!0,enumerable:!0,value:n},scale:{configurable:!0,enumerable:!0,value:s},modelViewMatrix:{value:new ce},normalMatrix:{value:new ee}}),this.matrix=new ce,this.matrixWorld=new ce,this.matrixAutoUpdate=i.DEFAULT_MATRIX_AUTO_UPDATE,this.matrixWorldAutoUpdate=i.DEFAULT_MATRIX_WORLD_AUTO_UPDATE,this.matrixWorldNeedsUpdate=!1,this.layers=new Fr,this.visible=!0,this.castShadow=!1,this.receiveShadow=!1,this.frustumCulled=!0,this.renderOrder=0,this.animations=[],this.userData={}}onBeforeShadow(){}onAfterShadow(){}onBeforeRender(){}onAfterRender(){}applyMatrix4(t){this.matrixAutoUpdate&&this.updateMatrix(),this.matrix.premultiply(t),this.matrix.decompose(this.position,this.quaternion,this.scale)}applyQuaternion(t){return this.quaternion.premultiply(t),this}setRotationFromAxisAngle(t,e){this.quaternion.setFromAxisAngle(t,e)}setRotationFromEuler(t){this.quaternion.setFromEuler(t,!0)}setRotationFromMatrix(t){this.quaternion.setFromRotationMatrix(t)}setRotationFromQuaternion(t){this.quaternion.copy(t)}rotateOnAxis(t,e){return Hi.setFromAxisAngle(t,e),this.quaternion.multiply(Hi),this}rotateOnWorldAxis(t,e){return Hi.setFromAxisAngle(t,e),this.quaternion.premultiply(Hi),this}rotateX(t){return this.rotateOnAxis(ll,t)}rotateY(t){return this.rotateOnAxis(hl,t)}rotateZ(t){return this.rotateOnAxis(ul,t)}translateOnAxis(t,e){return cl.copy(t).applyQuaternion(this.quaternion),this.position.add(cl.multiplyScalar(e)),this}translateX(t){return this.translateOnAxis(ll,t)}translateY(t){return this.translateOnAxis(hl,t)}translateZ(t){return this.translateOnAxis(ul,t)}localToWorld(t){return this.updateWorldMatrix(!0,!1),t.applyMatrix4(this.matrixWorld)}worldToLocal(t){return this.updateWorldMatrix(!0,!1),t.applyMatrix4(Gn.copy(this.matrixWorld).invert())}lookAt(t,e,n){t.isVector3?rr.copy(t):rr.set(t,e,n);let s=this.parent;this.updateWorldMatrix(!0,!1),ws.setFromMatrixPosition(this.matrixWorld),this.isCamera||this.isLight?Gn.lookAt(ws,rr,this.up):Gn.lookAt(rr,ws,this.up),this.quaternion.setFromRotationMatrix(Gn),s&&(Gn.extractRotation(s.matrixWorld),Hi.setFromRotationMatrix(Gn),this.quaternion.premultiply(Hi.invert()))}add(t){if(arguments.length>1){for(let e=0;e<arguments.length;e++)this.add(arguments[e]);return this}return t===this?(console.error("THREE.Object3D.add: object can't be added as a child of itself.",t),this):(t&&t.isObject3D?(t.removeFromParent(),t.parent=this,this.children.push(t),t.dispatchEvent(dl),Vi.child=t,this.dispatchEvent(Vi),Vi.child=null):console.error("THREE.Object3D.add: object not an instance of THREE.Object3D.",t),this)}remove(t){if(arguments.length>1){for(let n=0;n<arguments.length;n++)this.remove(arguments[n]);return this}let e=this.children.indexOf(t);return e!==-1&&(t.parent=null,this.children.splice(e,1),t.dispatchEvent(Rd),Po.child=t,this.dispatchEvent(Po),Po.child=null),this}removeFromParent(){let t=this.parent;return t!==null&&t.remove(this),this}clear(){return this.remove(...this.children)}attach(t){return this.updateWorldMatrix(!0,!1),Gn.copy(this.matrixWorld).invert(),t.parent!==null&&(t.parent.updateWorldMatrix(!0,!1),Gn.multiply(t.parent.matrixWorld)),t.applyMatrix4(Gn),t.removeFromParent(),t.parent=this,this.children.push(t),t.updateWorldMatrix(!1,!0),t.dispatchEvent(dl),Vi.child=t,this.dispatchEvent(Vi),Vi.child=null,this}getObjectById(t){return this.getObjectByProperty("id",t)}getObjectByName(t){return this.getObjectByProperty("name",t)}getObjectByProperty(t,e){if(this[t]===e)return this;for(let n=0,s=this.children.length;n<s;n++){let o=this.children[n].getObjectByProperty(t,e);if(o!==void 0)return o}}getObjectsByProperty(t,e,n=[]){this[t]===e&&n.push(this);let s=this.children;for(let r=0,o=s.length;r<o;r++)s[r].getObjectsByProperty(t,e,n);return n}getWorldPosition(t){return this.updateWorldMatrix(!0,!1),t.setFromMatrixPosition(this.matrixWorld)}getWorldQuaternion(t){return this.updateWorldMatrix(!0,!1),this.matrixWorld.decompose(ws,t,Ad),t}getWorldScale(t){return this.updateWorldMatrix(!0,!1),this.matrixWorld.decompose(ws,Cd,t),t}getWorldDirection(t){this.updateWorldMatrix(!0,!1);let e=this.matrixWorld.elements;return t.set(e[8],e[9],e[10]).normalize()}raycast(){}traverse(t){t(this);let e=this.children;for(let n=0,s=e.length;n<s;n++)e[n].traverse(t)}traverseVisible(t){if(this.visible===!1)return;t(this);let e=this.children;for(let n=0,s=e.length;n<s;n++)e[n].traverseVisible(t)}traverseAncestors(t){let e=this.parent;e!==null&&(t(e),e.traverseAncestors(t))}updateMatrix(){this.matrix.compose(this.position,this.quaternion,this.scale),this.matrixWorldNeedsUpdate=!0}updateMatrixWorld(t){this.matrixAutoUpdate&&this.updateMatrix(),(this.matrixWorldNeedsUpdate||t)&&(this.matrixWorldAutoUpdate===!0&&(this.parent===null?this.matrixWorld.copy(this.matrix):this.matrixWorld.multiplyMatrices(this.parent.matrixWorld,this.matrix)),this.matrixWorldNeedsUpdate=!1,t=!0);let e=this.children;for(let n=0,s=e.length;n<s;n++)e[n].updateMatrixWorld(t)}updateWorldMatrix(t,e){let n=this.parent;if(t===!0&&n!==null&&n.updateWorldMatrix(!0,!1),this.matrixAutoUpdate&&this.updateMatrix(),this.matrixWorldAutoUpdate===!0&&(this.parent===null?this.matrixWorld.copy(this.matrix):this.matrixWorld.multiplyMatrices(this.parent.matrixWorld,this.matrix)),e===!0){let s=this.children;for(let r=0,o=s.length;r<o;r++)s[r].updateWorldMatrix(!1,!0)}}toJSON(t){let e=t===void 0||typeof t=="string",n={};e&&(t={geometries:{},materials:{},textures:{},images:{},shapes:{},skeletons:{},animations:{},nodes:{}},n.metadata={version:4.6,type:"Object",generator:"Object3D.toJSON"});let s={};s.uuid=this.uuid,s.type=this.type,this.name!==""&&(s.name=this.name),this.castShadow===!0&&(s.castShadow=!0),this.receiveShadow===!0&&(s.receiveShadow=!0),this.visible===!1&&(s.visible=!1),this.frustumCulled===!1&&(s.frustumCulled=!1),this.renderOrder!==0&&(s.renderOrder=this.renderOrder),Object.keys(this.userData).length>0&&(s.userData=this.userData),s.layers=this.layers.mask,s.matrix=this.matrix.toArray(),s.up=this.up.toArray(),this.matrixAutoUpdate===!1&&(s.matrixAutoUpdate=!1),this.isInstancedMesh&&(s.type="InstancedMesh",s.count=this.count,s.instanceMatrix=this.instanceMatrix.toJSON(),this.instanceColor!==null&&(s.instanceColor=this.instanceColor.toJSON())),this.isBatchedMesh&&(s.type="BatchedMesh",s.perObjectFrustumCulled=this.perObjectFrustumCulled,s.sortObjects=this.sortObjects,s.drawRanges=this._drawRanges,s.reservedRanges=this._reservedRanges,s.visibility=this._visibility,s.active=this._active,s.bounds=this._bounds.map(a=>({boxInitialized:a.boxInitialized,boxMin:a.box.min.toArray(),boxMax:a.box.max.toArray(),sphereInitialized:a.sphereInitialized,sphereRadius:a.sphere.radius,sphereCenter:a.sphere.center.toArray()})),s.maxInstanceCount=this._maxInstanceCount,s.maxVertexCount=this._maxVertexCount,s.maxIndexCount=this._maxIndexCount,s.geometryInitialized=this._geometryInitialized,s.geometryCount=this._geometryCount,s.matricesTexture=this._matricesTexture.toJSON(t),this._colorsTexture!==null&&(s.colorsTexture=this._colorsTexture.toJSON(t)),this.boundingSphere!==null&&(s.boundingSphere={center:s.boundingSphere.center.toArray(),radius:s.boundingSphere.radius}),this.boundingBox!==null&&(s.boundingBox={min:s.boundingBox.min.toArray(),max:s.boundingBox.max.toArray()}));function r(a,c){return a[c.uuid]===void 0&&(a[c.uuid]=c.toJSON(t)),c.uuid}if(this.isScene)this.background&&(this.background.isColor?s.background=this.background.toJSON():this.background.isTexture&&(s.background=this.background.toJSON(t).uuid)),this.environment&&this.environment.isTexture&&this.environment.isRenderTargetTexture!==!0&&(s.environment=this.environment.toJSON(t).uuid);else if(this.isMesh||this.isLine||this.isPoints){s.geometry=r(t.geometries,this.geometry);let a=this.geometry.parameters;if(a!==void 0&&a.shapes!==void 0){let c=a.shapes;if(Array.isArray(c))for(let l=0,h=c.length;l<h;l++){let u=c[l];r(t.shapes,u)}else r(t.shapes,c)}}if(this.isSkinnedMesh&&(s.bindMode=this.bindMode,s.bindMatrix=this.bindMatrix.toArray(),this.skeleton!==void 0&&(r(t.skeletons,this.skeleton),s.skeleton=this.skeleton.uuid)),this.material!==void 0)if(Array.isArray(this.material)){let a=[];for(let c=0,l=this.material.length;c<l;c++)a.push(r(t.materials,this.material[c]));s.material=a}else s.material=r(t.materials,this.material);if(this.children.length>0){s.children=[];for(let a=0;a<this.children.length;a++)s.children.push(this.children[a].toJSON(t).object)}if(this.animations.length>0){s.animations=[];for(let a=0;a<this.animations.length;a++){let c=this.animations[a];s.animations.push(r(t.animations,c))}}if(e){let a=o(t.geometries),c=o(t.materials),l=o(t.textures),h=o(t.images),u=o(t.shapes),p=o(t.skeletons),f=o(t.animations),g=o(t.nodes);a.length>0&&(n.geometries=a),c.length>0&&(n.materials=c),l.length>0&&(n.textures=l),h.length>0&&(n.images=h),u.length>0&&(n.shapes=u),p.length>0&&(n.skeletons=p),f.length>0&&(n.animations=f),g.length>0&&(n.nodes=g)}return n.object=s,n;function o(a){let c=[];for(let l in a){let h=a[l];delete h.metadata,c.push(h)}return c}}clone(t){return new this.constructor().copy(this,t)}copy(t,e=!0){if(this.name=t.name,this.up.copy(t.up),this.position.copy(t.position),this.rotation.order=t.rotation.order,this.quaternion.copy(t.quaternion),this.scale.copy(t.scale),this.matrix.copy(t.matrix),this.matrixWorld.copy(t.matrixWorld),this.matrixAutoUpdate=t.matrixAutoUpdate,this.matrixWorldAutoUpdate=t.matrixWorldAutoUpdate,this.matrixWorldNeedsUpdate=t.matrixWorldNeedsUpdate,this.layers.mask=t.layers.mask,this.visible=t.visible,this.castShadow=t.castShadow,this.receiveShadow=t.receiveShadow,this.frustumCulled=t.frustumCulled,this.renderOrder=t.renderOrder,this.animations=t.animations.slice(),this.userData=JSON.parse(JSON.stringify(t.userData)),e===!0)for(let n=0;n<t.children.length;n++){let s=t.children[n];this.add(s.clone())}return this}};Re.DEFAULT_UP=new U(0,1,0);Re.DEFAULT_MATRIX_AUTO_UPDATE=!0;Re.DEFAULT_MATRIX_WORLD_AUTO_UPDATE=!0;var Sn=new U,Wn=new U,Io=new U,Xn=new U,Gi=new U,Wi=new U,fl=new U,Lo=new U,Do=new U,Uo=new U,No=new be,Fo=new be,Oo=new be,ui=class i{constructor(t=new U,e=new U,n=new U){this.a=t,this.b=e,this.c=n}static getNormal(t,e,n,s){s.subVectors(n,e),Sn.subVectors(t,e),s.cross(Sn);let r=s.lengthSq();return r>0?s.multiplyScalar(1/Math.sqrt(r)):s.set(0,0,0)}static getBarycoord(t,e,n,s,r){Sn.subVectors(s,e),Wn.subVectors(n,e),Io.subVectors(t,e);let o=Sn.dot(Sn),a=Sn.dot(Wn),c=Sn.dot(Io),l=Wn.dot(Wn),h=Wn.dot(Io),u=o*l-a*a;if(u===0)return r.set(0,0,0),null;let p=1/u,f=(l*c-a*h)*p,g=(o*h-a*c)*p;return r.set(1-f-g,g,f)}static containsPoint(t,e,n,s){return this.getBarycoord(t,e,n,s,Xn)===null?!1:Xn.x>=0&&Xn.y>=0&&Xn.x+Xn.y<=1}static getInterpolation(t,e,n,s,r,o,a,c){return this.getBarycoord(t,e,n,s,Xn)===null?(c.x=0,c.y=0,"z"in c&&(c.z=0),"w"in c&&(c.w=0),null):(c.setScalar(0),c.addScaledVector(r,Xn.x),c.addScaledVector(o,Xn.y),c.addScaledVector(a,Xn.z),c)}static getInterpolatedAttribute(t,e,n,s,r,o){return No.setScalar(0),Fo.setScalar(0),Oo.setScalar(0),No.fromBufferAttribute(t,e),Fo.fromBufferAttribute(t,n),Oo.fromBufferAttribute(t,s),o.setScalar(0),o.addScaledVector(No,r.x),o.addScaledVector(Fo,r.y),o.addScaledVector(Oo,r.z),o}static isFrontFacing(t,e,n,s){return Sn.subVectors(n,e),Wn.subVectors(t,e),Sn.cross(Wn).dot(s)<0}set(t,e,n){return this.a.copy(t),this.b.copy(e),this.c.copy(n),this}setFromPointsAndIndices(t,e,n,s){return this.a.copy(t[e]),this.b.copy(t[n]),this.c.copy(t[s]),this}setFromAttributeAndIndices(t,e,n,s){return this.a.fromBufferAttribute(t,e),this.b.fromBufferAttribute(t,n),this.c.fromBufferAttribute(t,s),this}clone(){return new this.constructor().copy(this)}copy(t){return this.a.copy(t.a),this.b.copy(t.b),this.c.copy(t.c),this}getArea(){return Sn.subVectors(this.c,this.b),Wn.subVectors(this.a,this.b),Sn.cross(Wn).length()*.5}getMidpoint(t){return t.addVectors(this.a,this.b).add(this.c).multiplyScalar(1/3)}getNormal(t){return i.getNormal(this.a,this.b,this.c,t)}getPlane(t){return t.setFromCoplanarPoints(this.a,this.b,this.c)}getBarycoord(t,e){return i.getBarycoord(t,this.a,this.b,this.c,e)}getInterpolation(t,e,n,s,r){return i.getInterpolation(t,this.a,this.b,this.c,e,n,s,r)}containsPoint(t){return i.containsPoint(t,this.a,this.b,this.c)}isFrontFacing(t){return i.isFrontFacing(this.a,this.b,this.c,t)}intersectsBox(t){return t.intersectsTriangle(this)}closestPointToPoint(t,e){let n=this.a,s=this.b,r=this.c,o,a;Gi.subVectors(s,n),Wi.subVectors(r,n),Lo.subVectors(t,n);let c=Gi.dot(Lo),l=Wi.dot(Lo);if(c<=0&&l<=0)return e.copy(n);Do.subVectors(t,s);let h=Gi.dot(Do),u=Wi.dot(Do);if(h>=0&&u<=h)return e.copy(s);let p=c*u-h*l;if(p<=0&&c>=0&&h<=0)return o=c/(c-h),e.copy(n).addScaledVector(Gi,o);Uo.subVectors(t,r);let f=Gi.dot(Uo),g=Wi.dot(Uo);if(g>=0&&f<=g)return e.copy(r);let y=f*l-c*g;if(y<=0&&l>=0&&g<=0)return a=l/(l-g),e.copy(n).addScaledVector(Wi,a);let m=h*g-f*u;if(m<=0&&u-h>=0&&f-g>=0)return fl.subVectors(r,s),a=(u-h)/(u-h+(f-g)),e.copy(s).addScaledVector(fl,a);let d=1/(m+y+p);return o=y*d,a=p*d,e.copy(n).addScaledVector(Gi,o).addScaledVector(Wi,a)}equals(t){return t.a.equals(this.a)&&t.b.equals(this.b)&&t.c.equals(this.c)}},uh={aliceblue:15792383,antiquewhite:16444375,aqua:65535,aquamarine:8388564,azure:15794175,beige:16119260,bisque:16770244,black:0,blanchedalmond:16772045,blue:255,blueviolet:9055202,brown:10824234,burlywood:14596231,cadetblue:6266528,chartreuse:8388352,chocolate:13789470,coral:16744272,cornflowerblue:6591981,cornsilk:16775388,crimson:14423100,cyan:65535,darkblue:139,darkcyan:35723,darkgoldenrod:12092939,darkgray:11119017,darkgreen:25600,darkgrey:11119017,darkkhaki:12433259,darkmagenta:9109643,darkolivegreen:5597999,darkorange:16747520,darkorchid:10040012,darkred:9109504,darksalmon:15308410,darkseagreen:9419919,darkslateblue:4734347,darkslategray:3100495,darkslategrey:3100495,darkturquoise:52945,darkviolet:9699539,deeppink:16716947,deepskyblue:49151,dimgray:6908265,dimgrey:6908265,dodgerblue:2003199,firebrick:11674146,floralwhite:16775920,forestgreen:2263842,fuchsia:16711935,gainsboro:14474460,ghostwhite:16316671,gold:16766720,goldenrod:14329120,gray:8421504,green:32768,greenyellow:11403055,grey:8421504,honeydew:15794160,hotpink:16738740,indianred:13458524,indigo:4915330,ivory:16777200,khaki:15787660,lavender:15132410,lavenderblush:16773365,lawngreen:8190976,lemonchiffon:16775885,lightblue:11393254,lightcoral:15761536,lightcyan:14745599,lightgoldenrodyellow:16448210,lightgray:13882323,lightgreen:9498256,lightgrey:13882323,lightpink:16758465,lightsalmon:16752762,lightseagreen:2142890,lightskyblue:8900346,lightslategray:7833753,lightslategrey:7833753,lightsteelblue:11584734,lightyellow:16777184,lime:65280,limegreen:3329330,linen:16445670,magenta:16711935,maroon:8388608,mediumaquamarine:6737322,mediumblue:205,mediumorchid:12211667,mediumpurple:9662683,mediumseagreen:3978097,mediumslateblue:8087790,mediumspringgreen:64154,mediumturquoise:4772300,mediumvioletred:13047173,midnightblue:1644912,mintcream:16121850,mistyrose:16770273,moccasin:16770229,navajowhite:16768685,navy:128,oldlace:16643558,olive:8421376,olivedrab:7048739,orange:16753920,orangered:16729344,orchid:14315734,palegoldenrod:15657130,palegreen:10025880,paleturquoise:11529966,palevioletred:14381203,papayawhip:16773077,peachpuff:16767673,peru:13468991,pink:16761035,plum:14524637,powderblue:11591910,purple:8388736,rebeccapurple:6697881,red:16711680,rosybrown:12357519,royalblue:4286945,saddlebrown:9127187,salmon:16416882,sandybrown:16032864,seagreen:3050327,seashell:16774638,sienna:10506797,silver:12632256,skyblue:8900331,slateblue:6970061,slategray:7372944,slategrey:7372944,snow:16775930,springgreen:65407,steelblue:4620980,tan:13808780,teal:32896,thistle:14204888,tomato:16737095,turquoise:4251856,violet:15631086,wheat:16113331,white:16777215,whitesmoke:16119285,yellow:16776960,yellowgreen:10145074},ci={h:0,s:0,l:0},or={h:0,s:0,l:0};function Bo(i,t,e){return e<0&&(e+=1),e>1&&(e-=1),e<1/6?i+(t-i)*6*e:e<1/2?t:e<2/3?i+(t-i)*6*(2/3-e):i}var _t=class{constructor(t,e,n){return this.isColor=!0,this.r=1,this.g=1,this.b=1,this.set(t,e,n)}set(t,e,n){if(e===void 0&&n===void 0){let s=t;s&&s.isColor?this.copy(s):typeof s=="number"?this.setHex(s):typeof s=="string"&&this.setStyle(s)}else this.setRGB(t,e,n);return this}setScalar(t){return this.r=t,this.g=t,this.b=t,this}setHex(t,e=we){return t=Math.floor(t),this.r=(t>>16&255)/255,this.g=(t>>8&255)/255,this.b=(t&255)/255,he.toWorkingColorSpace(this,e),this}setRGB(t,e,n,s=he.workingColorSpace){return this.r=t,this.g=e,this.b=n,he.toWorkingColorSpace(this,s),this}setHSL(t,e,n,s=he.workingColorSpace){if(t=Sc(t,1),e=Ye(e,0,1),n=Ye(n,0,1),e===0)this.r=this.g=this.b=n;else{let r=n<=.5?n*(1+e):n+e-n*e,o=2*n-r;this.r=Bo(o,r,t+1/3),this.g=Bo(o,r,t),this.b=Bo(o,r,t-1/3)}return he.toWorkingColorSpace(this,s),this}setStyle(t,e=we){function n(r){r!==void 0&&parseFloat(r)<1&&console.warn("THREE.Color: Alpha component of "+t+" will be ignored.")}let s;if(s=/^(\w+)\(([^\)]*)\)/.exec(t)){let r,o=s[1],a=s[2];switch(o){case"rgb":case"rgba":if(r=/^\s*(\d+)\s*,\s*(\d+)\s*,\s*(\d+)\s*(?:,\s*(\d*\.?\d+)\s*)?$/.exec(a))return n(r[4]),this.setRGB(Math.min(255,parseInt(r[1],10))/255,Math.min(255,parseInt(r[2],10))/255,Math.min(255,parseInt(r[3],10))/255,e);if(r=/^\s*(\d+)\%\s*,\s*(\d+)\%\s*,\s*(\d+)\%\s*(?:,\s*(\d*\.?\d+)\s*)?$/.exec(a))return n(r[4]),this.setRGB(Math.min(100,parseInt(r[1],10))/100,Math.min(100,parseInt(r[2],10))/100,Math.min(100,parseInt(r[3],10))/100,e);break;case"hsl":case"hsla":if(r=/^\s*(\d*\.?\d+)\s*,\s*(\d*\.?\d+)\%\s*,\s*(\d*\.?\d+)\%\s*(?:,\s*(\d*\.?\d+)\s*)?$/.exec(a))return n(r[4]),this.setHSL(parseFloat(r[1])/360,parseFloat(r[2])/100,parseFloat(r[3])/100,e);break;default:console.warn("THREE.Color: Unknown color model "+t)}}else if(s=/^\#([A-Fa-f\d]+)$/.exec(t)){let r=s[1],o=r.length;if(o===3)return this.setRGB(parseInt(r.charAt(0),16)/15,parseInt(r.charAt(1),16)/15,parseInt(r.charAt(2),16)/15,e);if(o===6)return this.setHex(parseInt(r,16),e);console.warn("THREE.Color: Invalid hex color "+t)}else if(t&&t.length>0)return this.setColorName(t,e);return this}setColorName(t,e=we){let n=uh[t.toLowerCase()];return n!==void 0?this.setHex(n,e):console.warn("THREE.Color: Unknown color "+t),this}clone(){return new this.constructor(this.r,this.g,this.b)}copy(t){return this.r=t.r,this.g=t.g,this.b=t.b,this}copySRGBToLinear(t){return this.r=Kn(t.r),this.g=Kn(t.g),this.b=Kn(t.b),this}copyLinearToSRGB(t){return this.r=is(t.r),this.g=is(t.g),this.b=is(t.b),this}convertSRGBToLinear(){return this.copySRGBToLinear(this),this}convertLinearToSRGB(){return this.copyLinearToSRGB(this),this}getHex(t=we){return he.fromWorkingColorSpace(Xe.copy(this),t),Math.round(Ye(Xe.r*255,0,255))*65536+Math.round(Ye(Xe.g*255,0,255))*256+Math.round(Ye(Xe.b*255,0,255))}getHexString(t=we){return("000000"+this.getHex(t).toString(16)).slice(-6)}getHSL(t,e=he.workingColorSpace){he.fromWorkingColorSpace(Xe.copy(this),e);let n=Xe.r,s=Xe.g,r=Xe.b,o=Math.max(n,s,r),a=Math.min(n,s,r),c,l,h=(a+o)/2;if(a===o)c=0,l=0;else{let u=o-a;switch(l=h<=.5?u/(o+a):u/(2-o-a),o){case n:c=(s-r)/u+(s<r?6:0);break;case s:c=(r-n)/u+2;break;case r:c=(n-s)/u+4;break}c/=6}return t.h=c,t.s=l,t.l=h,t}getRGB(t,e=he.workingColorSpace){return he.fromWorkingColorSpace(Xe.copy(this),e),t.r=Xe.r,t.g=Xe.g,t.b=Xe.b,t}getStyle(t=we){he.fromWorkingColorSpace(Xe.copy(this),t);let e=Xe.r,n=Xe.g,s=Xe.b;return t!==we?`color(${t} ${e.toFixed(3)} ${n.toFixed(3)} ${s.toFixed(3)})`:`rgb(${Math.round(e*255)},${Math.round(n*255)},${Math.round(s*255)})`}offsetHSL(t,e,n){return this.getHSL(ci),this.setHSL(ci.h+t,ci.s+e,ci.l+n)}add(t){return this.r+=t.r,this.g+=t.g,this.b+=t.b,this}addColors(t,e){return this.r=t.r+e.r,this.g=t.g+e.g,this.b=t.b+e.b,this}addScalar(t){return this.r+=t,this.g+=t,this.b+=t,this}sub(t){return this.r=Math.max(0,this.r-t.r),this.g=Math.max(0,this.g-t.g),this.b=Math.max(0,this.b-t.b),this}multiply(t){return this.r*=t.r,this.g*=t.g,this.b*=t.b,this}multiplyScalar(t){return this.r*=t,this.g*=t,this.b*=t,this}lerp(t,e){return this.r+=(t.r-this.r)*e,this.g+=(t.g-this.g)*e,this.b+=(t.b-this.b)*e,this}lerpColors(t,e,n){return this.r=t.r+(e.r-t.r)*n,this.g=t.g+(e.g-t.g)*n,this.b=t.b+(e.b-t.b)*n,this}lerpHSL(t,e){this.getHSL(ci),t.getHSL(or);let n=Fs(ci.h,or.h,e),s=Fs(ci.s,or.s,e),r=Fs(ci.l,or.l,e);return this.setHSL(n,s,r),this}setFromVector3(t){return this.r=t.x,this.g=t.y,this.b=t.z,this}applyMatrix3(t){let e=this.r,n=this.g,s=this.b,r=t.elements;return this.r=r[0]*e+r[3]*n+r[6]*s,this.g=r[1]*e+r[4]*n+r[7]*s,this.b=r[2]*e+r[5]*n+r[8]*s,this}equals(t){return t.r===this.r&&t.g===this.g&&t.b===this.b}fromArray(t,e=0){return this.r=t[e],this.g=t[e+1],this.b=t[e+2],this}toArray(t=[],e=0){return t[e]=this.r,t[e+1]=this.g,t[e+2]=this.b,t}fromBufferAttribute(t,e){return this.r=t.getX(e),this.g=t.getY(e),this.b=t.getZ(e),this}toJSON(){return this.getHex()}*[Symbol.iterator](){yield this.r,yield this.g,yield this.b}},Xe=new _t;_t.NAMES=uh;var Pd=0,Nn=class extends jn{static get type(){return"Material"}get type(){return this.constructor.type}set type(t){}constructor(){super(),this.isMaterial=!0,Object.defineProperty(this,"id",{value:Pd++}),this.uuid=Zn(),this.name="",this.blending=es,this.side=Dn,this.vertexColors=!1,this.opacity=1,this.transparent=!1,this.alphaHash=!1,this.blendSrc=Ko,this.blendDst=Jo,this.blendEquation=wi,this.blendSrcAlpha=null,this.blendDstAlpha=null,this.blendEquationAlpha=null,this.blendColor=new _t(0,0,0),this.blendAlpha=0,this.depthFunc=rs,this.depthTest=!0,this.depthWrite=!0,this.stencilWriteMask=255,this.stencilFunc=Kc,this.stencilRef=0,this.stencilFuncMask=255,this.stencilFail=Ni,this.stencilZFail=Ni,this.stencilZPass=Ni,this.stencilWrite=!1,this.clippingPlanes=null,this.clipIntersection=!1,this.clipShadows=!1,this.shadowSide=null,this.colorWrite=!0,this.precision=null,this.polygonOffset=!1,this.polygonOffsetFactor=0,this.polygonOffsetUnits=0,this.dithering=!1,this.alphaToCoverage=!1,this.premultipliedAlpha=!1,this.forceSinglePass=!1,this.visible=!0,this.toneMapped=!0,this.userData={},this.version=0,this._alphaTest=0}get alphaTest(){return this._alphaTest}set alphaTest(t){this._alphaTest>0!=t>0&&this.version++,this._alphaTest=t}onBeforeRender(){}onBeforeCompile(){}customProgramCacheKey(){return this.onBeforeCompile.toString()}setValues(t){if(t!==void 0)for(let e in t){let n=t[e];if(n===void 0){console.warn(`THREE.Material: parameter '${e}' has value of undefined.`);continue}let s=this[e];if(s===void 0){console.warn(`THREE.Material: '${e}' is not a property of THREE.${this.type}.`);continue}s&&s.isColor?s.set(n):s&&s.isVector3&&n&&n.isVector3?s.copy(n):this[e]=n}}toJSON(t){let e=t===void 0||typeof t=="string";e&&(t={textures:{},images:{}});let n={metadata:{version:4.6,type:"Material",generator:"Material.toJSON"}};n.uuid=this.uuid,n.type=this.type,this.name!==""&&(n.name=this.name),this.color&&this.color.isColor&&(n.color=this.color.getHex()),this.roughness!==void 0&&(n.roughness=this.roughness),this.metalness!==void 0&&(n.metalness=this.metalness),this.sheen!==void 0&&(n.sheen=this.sheen),this.sheenColor&&this.sheenColor.isColor&&(n.sheenColor=this.sheenColor.getHex()),this.sheenRoughness!==void 0&&(n.sheenRoughness=this.sheenRoughness),this.emissive&&this.emissive.isColor&&(n.emissive=this.emissive.getHex()),this.emissiveIntensity!==void 0&&this.emissiveIntensity!==1&&(n.emissiveIntensity=this.emissiveIntensity),this.specular&&this.specular.isColor&&(n.specular=this.specular.getHex()),this.specularIntensity!==void 0&&(n.specularIntensity=this.specularIntensity),this.specularColor&&this.specularColor.isColor&&(n.specularColor=this.specularColor.getHex()),this.shininess!==void 0&&(n.shininess=this.shininess),this.clearcoat!==void 0&&(n.clearcoat=this.clearcoat),this.clearcoatRoughness!==void 0&&(n.clearcoatRoughness=this.clearcoatRoughness),this.clearcoatMap&&this.clearcoatMap.isTexture&&(n.clearcoatMap=this.clearcoatMap.toJSON(t).uuid),this.clearcoatRoughnessMap&&this.clearcoatRoughnessMap.isTexture&&(n.clearcoatRoughnessMap=this.clearcoatRoughnessMap.toJSON(t).uuid),this.clearcoatNormalMap&&this.clearcoatNormalMap.isTexture&&(n.clearcoatNormalMap=this.clearcoatNormalMap.toJSON(t).uuid,n.clearcoatNormalScale=this.clearcoatNormalScale.toArray()),this.dispersion!==void 0&&(n.dispersion=this.dispersion),this.iridescence!==void 0&&(n.iridescence=this.iridescence),this.iridescenceIOR!==void 0&&(n.iridescenceIOR=this.iridescenceIOR),this.iridescenceThicknessRange!==void 0&&(n.iridescenceThicknessRange=this.iridescenceThicknessRange),this.iridescenceMap&&this.iridescenceMap.isTexture&&(n.iridescenceMap=this.iridescenceMap.toJSON(t).uuid),this.iridescenceThicknessMap&&this.iridescenceThicknessMap.isTexture&&(n.iridescenceThicknessMap=this.iridescenceThicknessMap.toJSON(t).uuid),this.anisotropy!==void 0&&(n.anisotropy=this.anisotropy),this.anisotropyRotation!==void 0&&(n.anisotropyRotation=this.anisotropyRotation),this.anisotropyMap&&this.anisotropyMap.isTexture&&(n.anisotropyMap=this.anisotropyMap.toJSON(t).uuid),this.map&&this.map.isTexture&&(n.map=this.map.toJSON(t).uuid),this.matcap&&this.matcap.isTexture&&(n.matcap=this.matcap.toJSON(t).uuid),this.alphaMap&&this.alphaMap.isTexture&&(n.alphaMap=this.alphaMap.toJSON(t).uuid),this.lightMap&&this.lightMap.isTexture&&(n.lightMap=this.lightMap.toJSON(t).uuid,n.lightMapIntensity=this.lightMapIntensity),this.aoMap&&this.aoMap.isTexture&&(n.aoMap=this.aoMap.toJSON(t).uuid,n.aoMapIntensity=this.aoMapIntensity),this.bumpMap&&this.bumpMap.isTexture&&(n.bumpMap=this.bumpMap.toJSON(t).uuid,n.bumpScale=this.bumpScale),this.normalMap&&this.normalMap.isTexture&&(n.normalMap=this.normalMap.toJSON(t).uuid,n.normalMapType=this.normalMapType,n.normalScale=this.normalScale.toArray()),this.displacementMap&&this.displacementMap.isTexture&&(n.displacementMap=this.displacementMap.toJSON(t).uuid,n.displacementScale=this.displacementScale,n.displacementBias=this.displacementBias),this.roughnessMap&&this.roughnessMap.isTexture&&(n.roughnessMap=this.roughnessMap.toJSON(t).uuid),this.metalnessMap&&this.metalnessMap.isTexture&&(n.metalnessMap=this.metalnessMap.toJSON(t).uuid),this.emissiveMap&&this.emissiveMap.isTexture&&(n.emissiveMap=this.emissiveMap.toJSON(t).uuid),this.specularMap&&this.specularMap.isTexture&&(n.specularMap=this.specularMap.toJSON(t).uuid),this.specularIntensityMap&&this.specularIntensityMap.isTexture&&(n.specularIntensityMap=this.specularIntensityMap.toJSON(t).uuid),this.specularColorMap&&this.specularColorMap.isTexture&&(n.specularColorMap=this.specularColorMap.toJSON(t).uuid),this.envMap&&this.envMap.isTexture&&(n.envMap=this.envMap.toJSON(t).uuid,this.combine!==void 0&&(n.combine=this.combine)),this.envMapRotation!==void 0&&(n.envMapRotation=this.envMapRotation.toArray()),this.envMapIntensity!==void 0&&(n.envMapIntensity=this.envMapIntensity),this.reflectivity!==void 0&&(n.reflectivity=this.reflectivity),this.refractionRatio!==void 0&&(n.refractionRatio=this.refractionRatio),this.gradientMap&&this.gradientMap.isTexture&&(n.gradientMap=this.gradientMap.toJSON(t).uuid),this.transmission!==void 0&&(n.transmission=this.transmission),this.transmissionMap&&this.transmissionMap.isTexture&&(n.transmissionMap=this.transmissionMap.toJSON(t).uuid),this.thickness!==void 0&&(n.thickness=this.thickness),this.thicknessMap&&this.thicknessMap.isTexture&&(n.thicknessMap=this.thicknessMap.toJSON(t).uuid),this.attenuationDistance!==void 0&&this.attenuationDistance!==1/0&&(n.attenuationDistance=this.attenuationDistance),this.attenuationColor!==void 0&&(n.attenuationColor=this.attenuationColor.getHex()),this.size!==void 0&&(n.size=this.size),this.shadowSide!==null&&(n.shadowSide=this.shadowSide),this.sizeAttenuation!==void 0&&(n.sizeAttenuation=this.sizeAttenuation),this.blending!==es&&(n.blending=this.blending),this.side!==Dn&&(n.side=this.side),this.vertexColors===!0&&(n.vertexColors=!0),this.opacity<1&&(n.opacity=this.opacity),this.transparent===!0&&(n.transparent=!0),this.blendSrc!==Ko&&(n.blendSrc=this.blendSrc),this.blendDst!==Jo&&(n.blendDst=this.blendDst),this.blendEquation!==wi&&(n.blendEquation=this.blendEquation),this.blendSrcAlpha!==null&&(n.blendSrcAlpha=this.blendSrcAlpha),this.blendDstAlpha!==null&&(n.blendDstAlpha=this.blendDstAlpha),this.blendEquationAlpha!==null&&(n.blendEquationAlpha=this.blendEquationAlpha),this.blendColor&&this.blendColor.isColor&&(n.blendColor=this.blendColor.getHex()),this.blendAlpha!==0&&(n.blendAlpha=this.blendAlpha),this.depthFunc!==rs&&(n.depthFunc=this.depthFunc),this.depthTest===!1&&(n.depthTest=this.depthTest),this.depthWrite===!1&&(n.depthWrite=this.depthWrite),this.colorWrite===!1&&(n.colorWrite=this.colorWrite),this.stencilWriteMask!==255&&(n.stencilWriteMask=this.stencilWriteMask),this.stencilFunc!==Kc&&(n.stencilFunc=this.stencilFunc),this.stencilRef!==0&&(n.stencilRef=this.stencilRef),this.stencilFuncMask!==255&&(n.stencilFuncMask=this.stencilFuncMask),this.stencilFail!==Ni&&(n.stencilFail=this.stencilFail),this.stencilZFail!==Ni&&(n.stencilZFail=this.stencilZFail),this.stencilZPass!==Ni&&(n.stencilZPass=this.stencilZPass),this.stencilWrite===!0&&(n.stencilWrite=this.stencilWrite),this.rotation!==void 0&&this.rotation!==0&&(n.rotation=this.rotation),this.polygonOffset===!0&&(n.polygonOffset=!0),this.polygonOffsetFactor!==0&&(n.polygonOffsetFactor=this.polygonOffsetFactor),this.polygonOffsetUnits!==0&&(n.polygonOffsetUnits=this.polygonOffsetUnits),this.linewidth!==void 0&&this.linewidth!==1&&(n.linewidth=this.linewidth),this.dashSize!==void 0&&(n.dashSize=this.dashSize),this.gapSize!==void 0&&(n.gapSize=this.gapSize),this.scale!==void 0&&(n.scale=this.scale),this.dithering===!0&&(n.dithering=!0),this.alphaTest>0&&(n.alphaTest=this.alphaTest),this.alphaHash===!0&&(n.alphaHash=!0),this.alphaToCoverage===!0&&(n.alphaToCoverage=!0),this.premultipliedAlpha===!0&&(n.premultipliedAlpha=!0),this.forceSinglePass===!0&&(n.forceSinglePass=!0),this.wireframe===!0&&(n.wireframe=!0),this.wireframeLinewidth>1&&(n.wireframeLinewidth=this.wireframeLinewidth),this.wireframeLinecap!=="round"&&(n.wireframeLinecap=this.wireframeLinecap),this.wireframeLinejoin!=="round"&&(n.wireframeLinejoin=this.wireframeLinejoin),this.flatShading===!0&&(n.flatShading=!0),this.visible===!1&&(n.visible=!1),this.toneMapped===!1&&(n.toneMapped=!1),this.fog===!1&&(n.fog=!1),Object.keys(this.userData).length>0&&(n.userData=this.userData);function s(r){let o=[];for(let a in r){let c=r[a];delete c.metadata,o.push(c)}return o}if(e){let r=s(t.textures),o=s(t.images);r.length>0&&(n.textures=r),o.length>0&&(n.images=o)}return n}clone(){return new this.constructor().copy(this)}copy(t){this.name=t.name,this.blending=t.blending,this.side=t.side,this.vertexColors=t.vertexColors,this.opacity=t.opacity,this.transparent=t.transparent,this.blendSrc=t.blendSrc,this.blendDst=t.blendDst,this.blendEquation=t.blendEquation,this.blendSrcAlpha=t.blendSrcAlpha,this.blendDstAlpha=t.blendDstAlpha,this.blendEquationAlpha=t.blendEquationAlpha,this.blendColor.copy(t.blendColor),this.blendAlpha=t.blendAlpha,this.depthFunc=t.depthFunc,this.depthTest=t.depthTest,this.depthWrite=t.depthWrite,this.stencilWriteMask=t.stencilWriteMask,this.stencilFunc=t.stencilFunc,this.stencilRef=t.stencilRef,this.stencilFuncMask=t.stencilFuncMask,this.stencilFail=t.stencilFail,this.stencilZFail=t.stencilZFail,this.stencilZPass=t.stencilZPass,this.stencilWrite=t.stencilWrite;let e=t.clippingPlanes,n=null;if(e!==null){let s=e.length;n=new Array(s);for(let r=0;r!==s;++r)n[r]=e[r].clone()}return this.clippingPlanes=n,this.clipIntersection=t.clipIntersection,this.clipShadows=t.clipShadows,this.shadowSide=t.shadowSide,this.colorWrite=t.colorWrite,this.precision=t.precision,this.polygonOffset=t.polygonOffset,this.polygonOffsetFactor=t.polygonOffsetFactor,this.polygonOffsetUnits=t.polygonOffsetUnits,this.dithering=t.dithering,this.alphaTest=t.alphaTest,this.alphaHash=t.alphaHash,this.alphaToCoverage=t.alphaToCoverage,this.premultipliedAlpha=t.premultipliedAlpha,this.forceSinglePass=t.forceSinglePass,this.visible=t.visible,this.toneMapped=t.toneMapped,this.userData=JSON.parse(JSON.stringify(t.userData)),this}dispose(){this.dispatchEvent({type:"dispose"})}set needsUpdate(t){t===!0&&this.version++}onBuild(){console.warn("Material: onBuild() has been removed.")}},Tn=class extends Nn{static get type(){return"MeshBasicMaterial"}constructor(t){super(),this.isMeshBasicMaterial=!0,this.color=new _t(16777215),this.map=null,this.lightMap=null,this.lightMapIntensity=1,this.aoMap=null,this.aoMapIntensity=1,this.specularMap=null,this.alphaMap=null,this.envMap=null,this.envMapRotation=new sn,this.combine=fc,this.reflectivity=1,this.refractionRatio=.98,this.wireframe=!1,this.wireframeLinewidth=1,this.wireframeLinecap="round",this.wireframeLinejoin="round",this.fog=!0,this.setValues(t)}copy(t){return super.copy(t),this.color.copy(t.color),this.map=t.map,this.lightMap=t.lightMap,this.lightMapIntensity=t.lightMapIntensity,this.aoMap=t.aoMap,this.aoMapIntensity=t.aoMapIntensity,this.specularMap=t.specularMap,this.alphaMap=t.alphaMap,this.envMap=t.envMap,this.envMapRotation.copy(t.envMapRotation),this.combine=t.combine,this.reflectivity=t.reflectivity,this.refractionRatio=t.refractionRatio,this.wireframe=t.wireframe,this.wireframeLinewidth=t.wireframeLinewidth,this.wireframeLinecap=t.wireframeLinecap,this.wireframeLinejoin=t.wireframeLinejoin,this.fog=t.fog,this}};var Te=new U,ar=new Wt,ve=class{constructor(t,e,n=!1){if(Array.isArray(t))throw new TypeError("THREE.BufferAttribute: array should be a Typed Array.");this.isBufferAttribute=!0,this.name="",this.array=t,this.itemSize=e,this.count=t!==void 0?t.length/e:0,this.normalized=n,this.usage=Na,this.updateRanges=[],this.gpuType=_n,this.version=0}onUploadCallback(){}set needsUpdate(t){t===!0&&this.version++}setUsage(t){return this.usage=t,this}addUpdateRange(t,e){this.updateRanges.push({start:t,count:e})}clearUpdateRanges(){this.updateRanges.length=0}copy(t){return this.name=t.name,this.array=new t.array.constructor(t.array),this.itemSize=t.itemSize,this.count=t.count,this.normalized=t.normalized,this.usage=t.usage,this.gpuType=t.gpuType,this}copyAt(t,e,n){t*=this.itemSize,n*=e.itemSize;for(let s=0,r=this.itemSize;s<r;s++)this.array[t+s]=e.array[n+s];return this}copyArray(t){return this.array.set(t),this}applyMatrix3(t){if(this.itemSize===2)for(let e=0,n=this.count;e<n;e++)ar.fromBufferAttribute(this,e),ar.applyMatrix3(t),this.setXY(e,ar.x,ar.y);else if(this.itemSize===3)for(let e=0,n=this.count;e<n;e++)Te.fromBufferAttribute(this,e),Te.applyMatrix3(t),this.setXYZ(e,Te.x,Te.y,Te.z);return this}applyMatrix4(t){for(let e=0,n=this.count;e<n;e++)Te.fromBufferAttribute(this,e),Te.applyMatrix4(t),this.setXYZ(e,Te.x,Te.y,Te.z);return this}applyNormalMatrix(t){for(let e=0,n=this.count;e<n;e++)Te.fromBufferAttribute(this,e),Te.applyNormalMatrix(t),this.setXYZ(e,Te.x,Te.y,Te.z);return this}transformDirection(t){for(let e=0,n=this.count;e<n;e++)Te.fromBufferAttribute(this,e),Te.transformDirection(t),this.setXYZ(e,Te.x,Te.y,Te.z);return this}set(t,e=0){return this.array.set(t,e),this}getComponent(t,e){let n=this.array[t*this.itemSize+e];return this.normalized&&(n=bn(n,this.array)),n}setComponent(t,e,n){return this.normalized&&(n=ge(n,this.array)),this.array[t*this.itemSize+e]=n,this}getX(t){let e=this.array[t*this.itemSize];return this.normalized&&(e=bn(e,this.array)),e}setX(t,e){return this.normalized&&(e=ge(e,this.array)),this.array[t*this.itemSize]=e,this}getY(t){let e=this.array[t*this.itemSize+1];return this.normalized&&(e=bn(e,this.array)),e}setY(t,e){return this.normalized&&(e=ge(e,this.array)),this.array[t*this.itemSize+1]=e,this}getZ(t){let e=this.array[t*this.itemSize+2];return this.normalized&&(e=bn(e,this.array)),e}setZ(t,e){return this.normalized&&(e=ge(e,this.array)),this.array[t*this.itemSize+2]=e,this}getW(t){let e=this.array[t*this.itemSize+3];return this.normalized&&(e=bn(e,this.array)),e}setW(t,e){return this.normalized&&(e=ge(e,this.array)),this.array[t*this.itemSize+3]=e,this}setXY(t,e,n){return t*=this.itemSize,this.normalized&&(e=ge(e,this.array),n=ge(n,this.array)),this.array[t+0]=e,this.array[t+1]=n,this}setXYZ(t,e,n,s){return t*=this.itemSize,this.normalized&&(e=ge(e,this.array),n=ge(n,this.array),s=ge(s,this.array)),this.array[t+0]=e,this.array[t+1]=n,this.array[t+2]=s,this}setXYZW(t,e,n,s,r){return t*=this.itemSize,this.normalized&&(e=ge(e,this.array),n=ge(n,this.array),s=ge(s,this.array),r=ge(r,this.array)),this.array[t+0]=e,this.array[t+1]=n,this.array[t+2]=s,this.array[t+3]=r,this}onUpload(t){return this.onUploadCallback=t,this}clone(){return new this.constructor(this.array,this.itemSize).copy(this)}toJSON(){let t={itemSize:this.itemSize,type:this.array.constructor.name,array:Array.from(this.array),normalized:this.normalized};return this.name!==""&&(t.name=this.name),this.usage!==Na&&(t.usage=this.usage),t}};var Or=class extends ve{constructor(t,e,n){super(new Uint16Array(t),e,n)}};var Br=class extends ve{constructor(t,e,n){super(new Uint32Array(t),e,n)}};var Ce=class extends ve{constructor(t,e,n){super(new Float32Array(t),e,n)}},Id=0,gn=new ce,zo=new Re,Xi=new U,hn=new Qn,As=new Qn,Be=new U,Ge=class i extends jn{constructor(){super(),this.isBufferGeometry=!0,Object.defineProperty(this,"id",{value:Id++}),this.uuid=Zn(),this.name="",this.type="BufferGeometry",this.index=null,this.indirect=null,this.attributes={},this.morphAttributes={},this.morphTargetsRelative=!1,this.groups=[],this.boundingBox=null,this.boundingSphere=null,this.drawRange={start:0,count:1/0},this.userData={}}getIndex(){return this.index}setIndex(t){return Array.isArray(t)?this.index=new(hh(t)?Br:Or)(t,1):this.index=t,this}setIndirect(t){return this.indirect=t,this}getIndirect(){return this.indirect}getAttribute(t){return this.attributes[t]}setAttribute(t,e){return this.attributes[t]=e,this}deleteAttribute(t){return delete this.attributes[t],this}hasAttribute(t){return this.attributes[t]!==void 0}addGroup(t,e,n=0){this.groups.push({start:t,count:e,materialIndex:n})}clearGroups(){this.groups=[]}setDrawRange(t,e){this.drawRange.start=t,this.drawRange.count=e}applyMatrix4(t){let e=this.attributes.position;e!==void 0&&(e.applyMatrix4(t),e.needsUpdate=!0);let n=this.attributes.normal;if(n!==void 0){let r=new ee().getNormalMatrix(t);n.applyNormalMatrix(r),n.needsUpdate=!0}let s=this.attributes.tangent;return s!==void 0&&(s.transformDirection(t),s.needsUpdate=!0),this.boundingBox!==null&&this.computeBoundingBox(),this.boundingSphere!==null&&this.computeBoundingSphere(),this}applyQuaternion(t){return gn.makeRotationFromQuaternion(t),this.applyMatrix4(gn),this}rotateX(t){return gn.makeRotationX(t),this.applyMatrix4(gn),this}rotateY(t){return gn.makeRotationY(t),this.applyMatrix4(gn),this}rotateZ(t){return gn.makeRotationZ(t),this.applyMatrix4(gn),this}translate(t,e,n){return gn.makeTranslation(t,e,n),this.applyMatrix4(gn),this}scale(t,e,n){return gn.makeScale(t,e,n),this.applyMatrix4(gn),this}lookAt(t){return zo.lookAt(t),zo.updateMatrix(),this.applyMatrix4(zo.matrix),this}center(){return this.computeBoundingBox(),this.boundingBox.getCenter(Xi).negate(),this.translate(Xi.x,Xi.y,Xi.z),this}setFromPoints(t){let e=this.getAttribute("position");if(e===void 0){let n=[];for(let s=0,r=t.length;s<r;s++){let o=t[s];n.push(o.x,o.y,o.z||0)}this.setAttribute("position",new Ce(n,3))}else{for(let n=0,s=e.count;n<s;n++){let r=t[n];e.setXYZ(n,r.x,r.y,r.z||0)}t.length>e.count&&console.warn("THREE.BufferGeometry: Buffer size too small for points data. Use .dispose() and create a new geometry."),e.needsUpdate=!0}return this}computeBoundingBox(){this.boundingBox===null&&(this.boundingBox=new Qn);let t=this.attributes.position,e=this.morphAttributes.position;if(t&&t.isGLBufferAttribute){console.error("THREE.BufferGeometry.computeBoundingBox(): GLBufferAttribute requires a manual bounding box.",this),this.boundingBox.set(new U(-1/0,-1/0,-1/0),new U(1/0,1/0,1/0));return}if(t!==void 0){if(this.boundingBox.setFromBufferAttribute(t),e)for(let n=0,s=e.length;n<s;n++){let r=e[n];hn.setFromBufferAttribute(r),this.morphTargetsRelative?(Be.addVectors(this.boundingBox.min,hn.min),this.boundingBox.expandByPoint(Be),Be.addVectors(this.boundingBox.max,hn.max),this.boundingBox.expandByPoint(Be)):(this.boundingBox.expandByPoint(hn.min),this.boundingBox.expandByPoint(hn.max))}}else this.boundingBox.makeEmpty();(isNaN(this.boundingBox.min.x)||isNaN(this.boundingBox.min.y)||isNaN(this.boundingBox.min.z))&&console.error('THREE.BufferGeometry.computeBoundingBox(): Computed min/max have NaN values. The "position" attribute is likely to have NaN values.',this)}computeBoundingSphere(){this.boundingSphere===null&&(this.boundingSphere=new mi);let t=this.attributes.position,e=this.morphAttributes.position;if(t&&t.isGLBufferAttribute){console.error("THREE.BufferGeometry.computeBoundingSphere(): GLBufferAttribute requires a manual bounding sphere.",this),this.boundingSphere.set(new U,1/0);return}if(t){let n=this.boundingSphere.center;if(hn.setFromBufferAttribute(t),e)for(let r=0,o=e.length;r<o;r++){let a=e[r];As.setFromBufferAttribute(a),this.morphTargetsRelative?(Be.addVectors(hn.min,As.min),hn.expandByPoint(Be),Be.addVectors(hn.max,As.max),hn.expandByPoint(Be)):(hn.expandByPoint(As.min),hn.expandByPoint(As.max))}hn.getCenter(n);let s=0;for(let r=0,o=t.count;r<o;r++)Be.fromBufferAttribute(t,r),s=Math.max(s,n.distanceToSquared(Be));if(e)for(let r=0,o=e.length;r<o;r++){let a=e[r],c=this.morphTargetsRelative;for(let l=0,h=a.count;l<h;l++)Be.fromBufferAttribute(a,l),c&&(Xi.fromBufferAttribute(t,l),Be.add(Xi)),s=Math.max(s,n.distanceToSquared(Be))}this.boundingSphere.radius=Math.sqrt(s),isNaN(this.boundingSphere.radius)&&console.error('THREE.BufferGeometry.computeBoundingSphere(): Computed radius is NaN. The "position" attribute is likely to have NaN values.',this)}}computeTangents(){let t=this.index,e=this.attributes;if(t===null||e.position===void 0||e.normal===void 0||e.uv===void 0){console.error("THREE.BufferGeometry: .computeTangents() failed. Missing required attributes (index, position, normal or uv)");return}let n=e.position,s=e.normal,r=e.uv;this.hasAttribute("tangent")===!1&&this.setAttribute("tangent",new ve(new Float32Array(4*n.count),4));let o=this.getAttribute("tangent"),a=[],c=[];for(let P=0;P<n.count;P++)a[P]=new U,c[P]=new U;let l=new U,h=new U,u=new U,p=new Wt,f=new Wt,g=new Wt,y=new U,m=new U;function d(P,E,M){l.fromBufferAttribute(n,P),h.fromBufferAttribute(n,E),u.fromBufferAttribute(n,M),p.fromBufferAttribute(r,P),f.fromBufferAttribute(r,E),g.fromBufferAttribute(r,M),h.sub(l),u.sub(l),f.sub(p),g.sub(p);let R=1/(f.x*g.y-g.x*f.y);isFinite(R)&&(y.copy(h).multiplyScalar(g.y).addScaledVector(u,-f.y).multiplyScalar(R),m.copy(u).multiplyScalar(f.x).addScaledVector(h,-g.x).multiplyScalar(R),a[P].add(y),a[E].add(y),a[M].add(y),c[P].add(m),c[E].add(m),c[M].add(m))}let x=this.groups;x.length===0&&(x=[{start:0,count:t.count}]);for(let P=0,E=x.length;P<E;++P){let M=x[P],R=M.start,F=M.count;for(let V=R,z=R+F;V<z;V+=3)d(t.getX(V+0),t.getX(V+1),t.getX(V+2))}let _=new U,v=new U,C=new U,w=new U;function A(P){C.fromBufferAttribute(s,P),w.copy(C);let E=a[P];_.copy(E),_.sub(C.multiplyScalar(C.dot(E))).normalize(),v.crossVectors(w,E);let R=v.dot(c[P])<0?-1:1;o.setXYZW(P,_.x,_.y,_.z,R)}for(let P=0,E=x.length;P<E;++P){let M=x[P],R=M.start,F=M.count;for(let V=R,z=R+F;V<z;V+=3)A(t.getX(V+0)),A(t.getX(V+1)),A(t.getX(V+2))}}computeVertexNormals(){let t=this.index,e=this.getAttribute("position");if(e!==void 0){let n=this.getAttribute("normal");if(n===void 0)n=new ve(new Float32Array(e.count*3),3),this.setAttribute("normal",n);else for(let p=0,f=n.count;p<f;p++)n.setXYZ(p,0,0,0);let s=new U,r=new U,o=new U,a=new U,c=new U,l=new U,h=new U,u=new U;if(t)for(let p=0,f=t.count;p<f;p+=3){let g=t.getX(p+0),y=t.getX(p+1),m=t.getX(p+2);s.fromBufferAttribute(e,g),r.fromBufferAttribute(e,y),o.fromBufferAttribute(e,m),h.subVectors(o,r),u.subVectors(s,r),h.cross(u),a.fromBufferAttribute(n,g),c.fromBufferAttribute(n,y),l.fromBufferAttribute(n,m),a.add(h),c.add(h),l.add(h),n.setXYZ(g,a.x,a.y,a.z),n.setXYZ(y,c.x,c.y,c.z),n.setXYZ(m,l.x,l.y,l.z)}else for(let p=0,f=e.count;p<f;p+=3)s.fromBufferAttribute(e,p+0),r.fromBufferAttribute(e,p+1),o.fromBufferAttribute(e,p+2),h.subVectors(o,r),u.subVectors(s,r),h.cross(u),n.setXYZ(p+0,h.x,h.y,h.z),n.setXYZ(p+1,h.x,h.y,h.z),n.setXYZ(p+2,h.x,h.y,h.z);this.normalizeNormals(),n.needsUpdate=!0}}normalizeNormals(){let t=this.attributes.normal;for(let e=0,n=t.count;e<n;e++)Be.fromBufferAttribute(t,e),Be.normalize(),t.setXYZ(e,Be.x,Be.y,Be.z)}toNonIndexed(){function t(a,c){let l=a.array,h=a.itemSize,u=a.normalized,p=new l.constructor(c.length*h),f=0,g=0;for(let y=0,m=c.length;y<m;y++){a.isInterleavedBufferAttribute?f=c[y]*a.data.stride+a.offset:f=c[y]*h;for(let d=0;d<h;d++)p[g++]=l[f++]}return new ve(p,h,u)}if(this.index===null)return console.warn("THREE.BufferGeometry.toNonIndexed(): BufferGeometry is already non-indexed."),this;let e=new i,n=this.index.array,s=this.attributes;for(let a in s){let c=s[a],l=t(c,n);e.setAttribute(a,l)}let r=this.morphAttributes;for(let a in r){let c=[],l=r[a];for(let h=0,u=l.length;h<u;h++){let p=l[h],f=t(p,n);c.push(f)}e.morphAttributes[a]=c}e.morphTargetsRelative=this.morphTargetsRelative;let o=this.groups;for(let a=0,c=o.length;a<c;a++){let l=o[a];e.addGroup(l.start,l.count,l.materialIndex)}return e}toJSON(){let t={metadata:{version:4.6,type:"BufferGeometry",generator:"BufferGeometry.toJSON"}};if(t.uuid=this.uuid,t.type=this.type,this.name!==""&&(t.name=this.name),Object.keys(this.userData).length>0&&(t.userData=this.userData),this.parameters!==void 0){let c=this.parameters;for(let l in c)c[l]!==void 0&&(t[l]=c[l]);return t}t.data={attributes:{}};let e=this.index;e!==null&&(t.data.index={type:e.array.constructor.name,array:Array.prototype.slice.call(e.array)});let n=this.attributes;for(let c in n){let l=n[c];t.data.attributes[c]=l.toJSON(t.data)}let s={},r=!1;for(let c in this.morphAttributes){let l=this.morphAttributes[c],h=[];for(let u=0,p=l.length;u<p;u++){let f=l[u];h.push(f.toJSON(t.data))}h.length>0&&(s[c]=h,r=!0)}r&&(t.data.morphAttributes=s,t.data.morphTargetsRelative=this.morphTargetsRelative);let o=this.groups;o.length>0&&(t.data.groups=JSON.parse(JSON.stringify(o)));let a=this.boundingSphere;return a!==null&&(t.data.boundingSphere={center:a.center.toArray(),radius:a.radius}),t}clone(){return new this.constructor().copy(this)}copy(t){this.index=null,this.attributes={},this.morphAttributes={},this.groups=[],this.boundingBox=null,this.boundingSphere=null;let e={};this.name=t.name;let n=t.index;n!==null&&this.setIndex(n.clone(e));let s=t.attributes;for(let l in s){let h=s[l];this.setAttribute(l,h.clone(e))}let r=t.morphAttributes;for(let l in r){let h=[],u=r[l];for(let p=0,f=u.length;p<f;p++)h.push(u[p].clone(e));this.morphAttributes[l]=h}this.morphTargetsRelative=t.morphTargetsRelative;let o=t.groups;for(let l=0,h=o.length;l<h;l++){let u=o[l];this.addGroup(u.start,u.count,u.materialIndex)}let a=t.boundingBox;a!==null&&(this.boundingBox=a.clone());let c=t.boundingSphere;return c!==null&&(this.boundingSphere=c.clone()),this.drawRange.start=t.drawRange.start,this.drawRange.count=t.drawRange.count,this.userData=t.userData,this}dispose(){this.dispatchEvent({type:"dispose"})}},pl=new ce,Mi=new hs,cr=new mi,ml=new U,lr=new U,hr=new U,ur=new U,ko=new U,dr=new U,gl=new U,fr=new U,se=class extends Re{constructor(t=new Ge,e=new Tn){super(),this.isMesh=!0,this.type="Mesh",this.geometry=t,this.material=e,this.updateMorphTargets()}copy(t,e){return super.copy(t,e),t.morphTargetInfluences!==void 0&&(this.morphTargetInfluences=t.morphTargetInfluences.slice()),t.morphTargetDictionary!==void 0&&(this.morphTargetDictionary=Object.assign({},t.morphTargetDictionary)),this.material=Array.isArray(t.material)?t.material.slice():t.material,this.geometry=t.geometry,this}updateMorphTargets(){let e=this.geometry.morphAttributes,n=Object.keys(e);if(n.length>0){let s=e[n[0]];if(s!==void 0){this.morphTargetInfluences=[],this.morphTargetDictionary={};for(let r=0,o=s.length;r<o;r++){let a=s[r].name||String(r);this.morphTargetInfluences.push(0),this.morphTargetDictionary[a]=r}}}}getVertexPosition(t,e){let n=this.geometry,s=n.attributes.position,r=n.morphAttributes.position,o=n.morphTargetsRelative;e.fromBufferAttribute(s,t);let a=this.morphTargetInfluences;if(r&&a){dr.set(0,0,0);for(let c=0,l=r.length;c<l;c++){let h=a[c],u=r[c];h!==0&&(ko.fromBufferAttribute(u,t),o?dr.addScaledVector(ko,h):dr.addScaledVector(ko.sub(e),h))}e.add(dr)}return e}raycast(t,e){let n=this.geometry,s=this.material,r=this.matrixWorld;s!==void 0&&(n.boundingSphere===null&&n.computeBoundingSphere(),cr.copy(n.boundingSphere),cr.applyMatrix4(r),Mi.copy(t.ray).recast(t.near),!(cr.containsPoint(Mi.origin)===!1&&(Mi.intersectSphere(cr,ml)===null||Mi.origin.distanceToSquared(ml)>(t.far-t.near)**2))&&(pl.copy(r).invert(),Mi.copy(t.ray).applyMatrix4(pl),!(n.boundingBox!==null&&Mi.intersectsBox(n.boundingBox)===!1)&&this._computeIntersections(t,e,Mi)))}_computeIntersections(t,e,n){let s,r=this.geometry,o=this.material,a=r.index,c=r.attributes.position,l=r.attributes.uv,h=r.attributes.uv1,u=r.attributes.normal,p=r.groups,f=r.drawRange;if(a!==null)if(Array.isArray(o))for(let g=0,y=p.length;g<y;g++){let m=p[g],d=o[m.materialIndex],x=Math.max(m.start,f.start),_=Math.min(a.count,Math.min(m.start+m.count,f.start+f.count));for(let v=x,C=_;v<C;v+=3){let w=a.getX(v),A=a.getX(v+1),P=a.getX(v+2);s=pr(this,d,t,n,l,h,u,w,A,P),s&&(s.faceIndex=Math.floor(v/3),s.face.materialIndex=m.materialIndex,e.push(s))}}else{let g=Math.max(0,f.start),y=Math.min(a.count,f.start+f.count);for(let m=g,d=y;m<d;m+=3){let x=a.getX(m),_=a.getX(m+1),v=a.getX(m+2);s=pr(this,o,t,n,l,h,u,x,_,v),s&&(s.faceIndex=Math.floor(m/3),e.push(s))}}else if(c!==void 0)if(Array.isArray(o))for(let g=0,y=p.length;g<y;g++){let m=p[g],d=o[m.materialIndex],x=Math.max(m.start,f.start),_=Math.min(c.count,Math.min(m.start+m.count,f.start+f.count));for(let v=x,C=_;v<C;v+=3){let w=v,A=v+1,P=v+2;s=pr(this,d,t,n,l,h,u,w,A,P),s&&(s.faceIndex=Math.floor(v/3),s.face.materialIndex=m.materialIndex,e.push(s))}}else{let g=Math.max(0,f.start),y=Math.min(c.count,f.start+f.count);for(let m=g,d=y;m<d;m+=3){let x=m,_=m+1,v=m+2;s=pr(this,o,t,n,l,h,u,x,_,v),s&&(s.faceIndex=Math.floor(m/3),e.push(s))}}}};function Ld(i,t,e,n,s,r,o,a){let c;if(t.side===ke?c=n.intersectTriangle(o,r,s,!0,a):c=n.intersectTriangle(s,r,o,t.side===Dn,a),c===null)return null;fr.copy(a),fr.applyMatrix4(i.matrixWorld);let l=e.ray.origin.distanceTo(fr);return l<e.near||l>e.far?null:{distance:l,point:fr.clone(),object:i}}function pr(i,t,e,n,s,r,o,a,c,l){i.getVertexPosition(a,lr),i.getVertexPosition(c,hr),i.getVertexPosition(l,ur);let h=Ld(i,t,e,n,lr,hr,ur,gl);if(h){let u=new U;ui.getBarycoord(gl,lr,hr,ur,u),s&&(h.uv=ui.getInterpolatedAttribute(s,a,c,l,u,new Wt)),r&&(h.uv1=ui.getInterpolatedAttribute(r,a,c,l,u,new Wt)),o&&(h.normal=ui.getInterpolatedAttribute(o,a,c,l,u,new U),h.normal.dot(n.direction)>0&&h.normal.multiplyScalar(-1));let p={a,b:c,c:l,normal:new U,materialIndex:0};ui.getNormal(lr,hr,ur,p.normal),h.face=p,h.barycoord=u}return h}var Qe=class i extends Ge{constructor(t=1,e=1,n=1,s=1,r=1,o=1){super(),this.type="BoxGeometry",this.parameters={width:t,height:e,depth:n,widthSegments:s,heightSegments:r,depthSegments:o};let a=this;s=Math.floor(s),r=Math.floor(r),o=Math.floor(o);let c=[],l=[],h=[],u=[],p=0,f=0;g("z","y","x",-1,-1,n,e,t,o,r,0),g("z","y","x",1,-1,n,e,-t,o,r,1),g("x","z","y",1,1,t,n,e,s,o,2),g("x","z","y",1,-1,t,n,-e,s,o,3),g("x","y","z",1,-1,t,e,n,s,r,4),g("x","y","z",-1,-1,t,e,-n,s,r,5),this.setIndex(c),this.setAttribute("position",new Ce(l,3)),this.setAttribute("normal",new Ce(h,3)),this.setAttribute("uv",new Ce(u,2));function g(y,m,d,x,_,v,C,w,A,P,E){let M=v/A,R=C/P,F=v/2,V=C/2,z=w/2,k=A+1,I=P+1,Y=0,B=0,Q=new U;for(let ut=0;ut<I;ut++){let Et=ut*R-V;for(let zt=0;zt<k;zt++){let $t=zt*M-F;Q[y]=$t*x,Q[m]=Et*_,Q[d]=z,l.push(Q.x,Q.y,Q.z),Q[y]=0,Q[m]=0,Q[d]=w>0?1:-1,h.push(Q.x,Q.y,Q.z),u.push(zt/A),u.push(1-ut/P),Y+=1}}for(let ut=0;ut<P;ut++)for(let Et=0;Et<A;Et++){let zt=p+Et+k*ut,$t=p+Et+k*(ut+1),H=p+(Et+1)+k*(ut+1),st=p+(Et+1)+k*ut;c.push(zt,$t,st),c.push($t,H,st),B+=6}a.addGroup(f,B,E),f+=B,p+=Y}}copy(t){return super.copy(t),this.parameters=Object.assign({},t.parameters),this}static fromJSON(t){return new i(t.width,t.height,t.depth,t.widthSegments,t.heightSegments,t.depthSegments)}};function us(i){let t={};for(let e in i){t[e]={};for(let n in i[e]){let s=i[e][n];s&&(s.isColor||s.isMatrix3||s.isMatrix4||s.isVector2||s.isVector3||s.isVector4||s.isTexture||s.isQuaternion)?s.isRenderTargetTexture?(console.warn("UniformsUtils: Textures of render targets cannot be cloned via cloneUniforms() or mergeUniforms()."),t[e][n]=null):t[e][n]=s.clone():Array.isArray(s)?t[e][n]=s.slice():t[e][n]=s}}return t}function Je(i){let t={};for(let e=0;e<i.length;e++){let n=us(i[e]);for(let s in n)t[s]=n[s]}return t}function Dd(i){let t=[];for(let e=0;e<i.length;e++)t.push(i[e].clone());return t}function dh(i){let t=i.getRenderTarget();return t===null?i.outputColorSpace:t.isXRRenderTarget===!0?t.texture.colorSpace:he.workingColorSpace}var xs={clone:us,merge:Je},Ud=`void main() {
	gl_Position = projectionMatrix * modelViewMatrix * vec4( position, 1.0 );
}`,Nd=`void main() {
	gl_FragColor = vec4( 1.0, 0.0, 0.0, 1.0 );
}`,Pe=class extends Nn{static get type(){return"ShaderMaterial"}constructor(t){super(),this.isShaderMaterial=!0,this.defines={},this.uniforms={},this.uniformsGroups=[],this.vertexShader=Ud,this.fragmentShader=Nd,this.linewidth=1,this.wireframe=!1,this.wireframeLinewidth=1,this.fog=!1,this.lights=!1,this.clipping=!1,this.forceSinglePass=!0,this.extensions={clipCullDistance:!1,multiDraw:!1},this.defaultAttributeValues={color:[1,1,1],uv:[0,0],uv1:[0,0]},this.index0AttributeName=void 0,this.uniformsNeedUpdate=!1,this.glslVersion=null,t!==void 0&&this.setValues(t)}copy(t){return super.copy(t),this.fragmentShader=t.fragmentShader,this.vertexShader=t.vertexShader,this.uniforms=us(t.uniforms),this.uniformsGroups=Dd(t.uniformsGroups),this.defines=Object.assign({},t.defines),this.wireframe=t.wireframe,this.wireframeLinewidth=t.wireframeLinewidth,this.fog=t.fog,this.lights=t.lights,this.clipping=t.clipping,this.extensions=Object.assign({},t.extensions),this.glslVersion=t.glslVersion,this}toJSON(t){let e=super.toJSON(t);e.glslVersion=this.glslVersion,e.uniforms={};for(let s in this.uniforms){let o=this.uniforms[s].value;o&&o.isTexture?e.uniforms[s]={type:"t",value:o.toJSON(t).uuid}:o&&o.isColor?e.uniforms[s]={type:"c",value:o.getHex()}:o&&o.isVector2?e.uniforms[s]={type:"v2",value:o.toArray()}:o&&o.isVector3?e.uniforms[s]={type:"v3",value:o.toArray()}:o&&o.isVector4?e.uniforms[s]={type:"v4",value:o.toArray()}:o&&o.isMatrix3?e.uniforms[s]={type:"m3",value:o.toArray()}:o&&o.isMatrix4?e.uniforms[s]={type:"m4",value:o.toArray()}:e.uniforms[s]={value:o}}Object.keys(this.defines).length>0&&(e.defines=this.defines),e.vertexShader=this.vertexShader,e.fragmentShader=this.fragmentShader,e.lights=this.lights,e.clipping=this.clipping;let n={};for(let s in this.extensions)this.extensions[s]===!0&&(n[s]=!0);return Object.keys(n).length>0&&(e.extensions=n),e}},zr=class extends Re{constructor(){super(),this.isCamera=!0,this.type="Camera",this.matrixWorldInverse=new ce,this.projectionMatrix=new ce,this.projectionMatrixInverse=new ce,this.coordinateSystem=$n}copy(t,e){return super.copy(t,e),this.matrixWorldInverse.copy(t.matrixWorldInverse),this.projectionMatrix.copy(t.projectionMatrix),this.projectionMatrixInverse.copy(t.projectionMatrixInverse),this.coordinateSystem=t.coordinateSystem,this}getWorldDirection(t){return super.getWorldDirection(t).negate()}updateMatrixWorld(t){super.updateMatrixWorld(t),this.matrixWorldInverse.copy(this.matrixWorld).invert()}updateWorldMatrix(t,e){super.updateWorldMatrix(t,e),this.matrixWorldInverse.copy(this.matrixWorld).invert()}clone(){return new this.constructor().copy(this)}},li=new U,_l=new Wt,xl=new Wt,Ve=class extends zr{constructor(t=50,e=1,n=.1,s=2e3){super(),this.isPerspectiveCamera=!0,this.type="PerspectiveCamera",this.fov=t,this.zoom=1,this.near=n,this.far=s,this.focus=10,this.aspect=e,this.view=null,this.filmGauge=35,this.filmOffset=0,this.updateProjectionMatrix()}copy(t,e){return super.copy(t,e),this.fov=t.fov,this.zoom=t.zoom,this.near=t.near,this.far=t.far,this.focus=t.focus,this.aspect=t.aspect,this.view=t.view===null?null:Object.assign({},t.view),this.filmGauge=t.filmGauge,this.filmOffset=t.filmOffset,this}setFocalLength(t){let e=.5*this.getFilmHeight()/t;this.fov=zs*2*Math.atan(e),this.updateProjectionMatrix()}getFocalLength(){let t=Math.tan(Ns*.5*this.fov);return .5*this.getFilmHeight()/t}getEffectiveFOV(){return zs*2*Math.atan(Math.tan(Ns*.5*this.fov)/this.zoom)}getFilmWidth(){return this.filmGauge*Math.min(this.aspect,1)}getFilmHeight(){return this.filmGauge/Math.max(this.aspect,1)}getViewBounds(t,e,n){li.set(-1,-1,.5).applyMatrix4(this.projectionMatrixInverse),e.set(li.x,li.y).multiplyScalar(-t/li.z),li.set(1,1,.5).applyMatrix4(this.projectionMatrixInverse),n.set(li.x,li.y).multiplyScalar(-t/li.z)}getViewSize(t,e){return this.getViewBounds(t,_l,xl),e.subVectors(xl,_l)}setViewOffset(t,e,n,s,r,o){this.aspect=t/e,this.view===null&&(this.view={enabled:!0,fullWidth:1,fullHeight:1,offsetX:0,offsetY:0,width:1,height:1}),this.view.enabled=!0,this.view.fullWidth=t,this.view.fullHeight=e,this.view.offsetX=n,this.view.offsetY=s,this.view.width=r,this.view.height=o,this.updateProjectionMatrix()}clearViewOffset(){this.view!==null&&(this.view.enabled=!1),this.updateProjectionMatrix()}updateProjectionMatrix(){let t=this.near,e=t*Math.tan(Ns*.5*this.fov)/this.zoom,n=2*e,s=this.aspect*n,r=-.5*s,o=this.view;if(this.view!==null&&this.view.enabled){let c=o.fullWidth,l=o.fullHeight;r+=o.offsetX*s/c,e-=o.offsetY*n/l,s*=o.width/c,n*=o.height/l}let a=this.filmOffset;a!==0&&(r+=t*a/this.getFilmWidth()),this.projectionMatrix.makePerspective(r,r+s,e,e-n,t,this.far,this.coordinateSystem),this.projectionMatrixInverse.copy(this.projectionMatrix).invert()}toJSON(t){let e=super.toJSON(t);return e.object.fov=this.fov,e.object.zoom=this.zoom,e.object.near=this.near,e.object.far=this.far,e.object.focus=this.focus,e.object.aspect=this.aspect,this.view!==null&&(e.object.view=Object.assign({},this.view)),e.object.filmGauge=this.filmGauge,e.object.filmOffset=this.filmOffset,e}},Yi=-90,qi=1,za=class extends Re{constructor(t,e,n){super(),this.type="CubeCamera",this.renderTarget=n,this.coordinateSystem=null,this.activeMipmapLevel=0;let s=new Ve(Yi,qi,t,e);s.layers=this.layers,this.add(s);let r=new Ve(Yi,qi,t,e);r.layers=this.layers,this.add(r);let o=new Ve(Yi,qi,t,e);o.layers=this.layers,this.add(o);let a=new Ve(Yi,qi,t,e);a.layers=this.layers,this.add(a);let c=new Ve(Yi,qi,t,e);c.layers=this.layers,this.add(c);let l=new Ve(Yi,qi,t,e);l.layers=this.layers,this.add(l)}updateCoordinateSystem(){let t=this.coordinateSystem,e=this.children.concat(),[n,s,r,o,a,c]=e;for(let l of e)this.remove(l);if(t===$n)n.up.set(0,1,0),n.lookAt(1,0,0),s.up.set(0,1,0),s.lookAt(-1,0,0),r.up.set(0,0,-1),r.lookAt(0,1,0),o.up.set(0,0,1),o.lookAt(0,-1,0),a.up.set(0,1,0),a.lookAt(0,0,1),c.up.set(0,1,0),c.lookAt(0,0,-1);else if(t===Lr)n.up.set(0,-1,0),n.lookAt(-1,0,0),s.up.set(0,-1,0),s.lookAt(1,0,0),r.up.set(0,0,1),r.lookAt(0,1,0),o.up.set(0,0,-1),o.lookAt(0,-1,0),a.up.set(0,-1,0),a.lookAt(0,0,1),c.up.set(0,-1,0),c.lookAt(0,0,-1);else throw new Error("THREE.CubeCamera.updateCoordinateSystem(): Invalid coordinate system: "+t);for(let l of e)this.add(l),l.updateMatrixWorld()}update(t,e){this.parent===null&&this.updateMatrixWorld();let{renderTarget:n,activeMipmapLevel:s}=this;this.coordinateSystem!==t.coordinateSystem&&(this.coordinateSystem=t.coordinateSystem,this.updateCoordinateSystem());let[r,o,a,c,l,h]=this.children,u=t.getRenderTarget(),p=t.getActiveCubeFace(),f=t.getActiveMipmapLevel(),g=t.xr.enabled;t.xr.enabled=!1;let y=n.texture.generateMipmaps;n.texture.generateMipmaps=!1,t.setRenderTarget(n,0,s),t.render(e,r),t.setRenderTarget(n,1,s),t.render(e,o),t.setRenderTarget(n,2,s),t.render(e,a),t.setRenderTarget(n,3,s),t.render(e,c),t.setRenderTarget(n,4,s),t.render(e,l),n.texture.generateMipmaps=y,t.setRenderTarget(n,5,s),t.render(e,h),t.setRenderTarget(u,p,f),t.xr.enabled=g,n.texture.needsPMREMUpdate=!0}},kr=class extends nn{constructor(t,e,n,s,r,o,a,c,l,h){t=t!==void 0?t:[],e=e!==void 0?e:os,super(t,e,n,s,r,o,a,c,l,h),this.isCubeTexture=!0,this.flipY=!1}get images(){return this.image}set images(t){this.image=t}},ka=class extends En{constructor(t=1,e={}){super(t,t,e),this.isWebGLCubeRenderTarget=!0;let n={width:t,height:t,depth:1},s=[n,n,n,n,n,n];this.texture=new kr(s,e.mapping,e.wrapS,e.wrapT,e.magFilter,e.minFilter,e.format,e.type,e.anisotropy,e.colorSpace),this.texture.isRenderTargetTexture=!0,this.texture.generateMipmaps=e.generateMipmaps!==void 0?e.generateMipmaps:!1,this.texture.minFilter=e.minFilter!==void 0?e.minFilter:je}fromEquirectangularTexture(t,e){this.texture.type=e.type,this.texture.colorSpace=e.colorSpace,this.texture.generateMipmaps=e.generateMipmaps,this.texture.minFilter=e.minFilter,this.texture.magFilter=e.magFilter;let n={uniforms:{tEquirect:{value:null}},vertexShader:`

				varying vec3 vWorldDirection;

				vec3 transformDirection( in vec3 dir, in mat4 matrix ) {

					return normalize( ( matrix * vec4( dir, 0.0 ) ).xyz );

				}

				void main() {

					vWorldDirection = transformDirection( position, modelMatrix );

					#include <begin_vertex>
					#include <project_vertex>

				}
			`,fragmentShader:`

				uniform sampler2D tEquirect;

				varying vec3 vWorldDirection;

				#include <common>

				void main() {

					vec3 direction = normalize( vWorldDirection );

					vec2 sampleUV = equirectUv( direction );

					gl_FragColor = texture2D( tEquirect, sampleUV );

				}
			`},s=new Qe(5,5,5),r=new Pe({name:"CubemapFromEquirect",uniforms:us(n.uniforms),vertexShader:n.vertexShader,fragmentShader:n.fragmentShader,side:ke,blending:di});r.uniforms.tEquirect.value=e;let o=new se(s,r),a=e.minFilter;return e.minFilter===qn&&(e.minFilter=je),new za(1,10,this).update(t,o),e.minFilter=a,o.geometry.dispose(),o.material.dispose(),this}clear(t,e,n,s){let r=t.getRenderTarget();for(let o=0;o<6;o++)t.setRenderTarget(this,o),t.clear(e,n,s);t.setRenderTarget(r)}},Ho=new U,Fd=new U,Od=new ee,un=class{constructor(t=new U(1,0,0),e=0){this.isPlane=!0,this.normal=t,this.constant=e}set(t,e){return this.normal.copy(t),this.constant=e,this}setComponents(t,e,n,s){return this.normal.set(t,e,n),this.constant=s,this}setFromNormalAndCoplanarPoint(t,e){return this.normal.copy(t),this.constant=-e.dot(this.normal),this}setFromCoplanarPoints(t,e,n){let s=Ho.subVectors(n,e).cross(Fd.subVectors(t,e)).normalize();return this.setFromNormalAndCoplanarPoint(s,t),this}copy(t){return this.normal.copy(t.normal),this.constant=t.constant,this}normalize(){let t=1/this.normal.length();return this.normal.multiplyScalar(t),this.constant*=t,this}negate(){return this.constant*=-1,this.normal.negate(),this}distanceToPoint(t){return this.normal.dot(t)+this.constant}distanceToSphere(t){return this.distanceToPoint(t.center)-t.radius}projectPoint(t,e){return e.copy(t).addScaledVector(this.normal,-this.distanceToPoint(t))}intersectLine(t,e){let n=t.delta(Ho),s=this.normal.dot(n);if(s===0)return this.distanceToPoint(t.start)===0?e.copy(t.start):null;let r=-(t.start.dot(this.normal)+this.constant)/s;return r<0||r>1?null:e.copy(t.start).addScaledVector(n,r)}intersectsLine(t){let e=this.distanceToPoint(t.start),n=this.distanceToPoint(t.end);return e<0&&n>0||n<0&&e>0}intersectsBox(t){return t.intersectsPlane(this)}intersectsSphere(t){return t.intersectsPlane(this)}coplanarPoint(t){return t.copy(this.normal).multiplyScalar(-this.constant)}applyMatrix4(t,e){let n=e||Od.getNormalMatrix(t),s=this.coplanarPoint(Ho).applyMatrix4(t),r=this.normal.applyMatrix3(n).normalize();return this.constant=-s.dot(r),this}translate(t){return this.constant-=t.dot(this.normal),this}equals(t){return t.normal.equals(this.normal)&&t.constant===this.constant}clone(){return new this.constructor().copy(this)}},Si=new mi,mr=new U,ks=class{constructor(t=new un,e=new un,n=new un,s=new un,r=new un,o=new un){this.planes=[t,e,n,s,r,o]}set(t,e,n,s,r,o){let a=this.planes;return a[0].copy(t),a[1].copy(e),a[2].copy(n),a[3].copy(s),a[4].copy(r),a[5].copy(o),this}copy(t){let e=this.planes;for(let n=0;n<6;n++)e[n].copy(t.planes[n]);return this}setFromProjectionMatrix(t,e=$n){let n=this.planes,s=t.elements,r=s[0],o=s[1],a=s[2],c=s[3],l=s[4],h=s[5],u=s[6],p=s[7],f=s[8],g=s[9],y=s[10],m=s[11],d=s[12],x=s[13],_=s[14],v=s[15];if(n[0].setComponents(c-r,p-l,m-f,v-d).normalize(),n[1].setComponents(c+r,p+l,m+f,v+d).normalize(),n[2].setComponents(c+o,p+h,m+g,v+x).normalize(),n[3].setComponents(c-o,p-h,m-g,v-x).normalize(),n[4].setComponents(c-a,p-u,m-y,v-_).normalize(),e===$n)n[5].setComponents(c+a,p+u,m+y,v+_).normalize();else if(e===Lr)n[5].setComponents(a,u,y,_).normalize();else throw new Error("THREE.Frustum.setFromProjectionMatrix(): Invalid coordinate system: "+e);return this}intersectsObject(t){if(t.boundingSphere!==void 0)t.boundingSphere===null&&t.computeBoundingSphere(),Si.copy(t.boundingSphere).applyMatrix4(t.matrixWorld);else{let e=t.geometry;e.boundingSphere===null&&e.computeBoundingSphere(),Si.copy(e.boundingSphere).applyMatrix4(t.matrixWorld)}return this.intersectsSphere(Si)}intersectsSprite(t){return Si.center.set(0,0,0),Si.radius=.7071067811865476,Si.applyMatrix4(t.matrixWorld),this.intersectsSphere(Si)}intersectsSphere(t){let e=this.planes,n=t.center,s=-t.radius;for(let r=0;r<6;r++)if(e[r].distanceToPoint(n)<s)return!1;return!0}intersectsBox(t){let e=this.planes;for(let n=0;n<6;n++){let s=e[n];if(mr.x=s.normal.x>0?t.max.x:t.min.x,mr.y=s.normal.y>0?t.max.y:t.min.y,mr.z=s.normal.z>0?t.max.z:t.min.z,s.distanceToPoint(mr)<0)return!1}return!0}containsPoint(t){let e=this.planes;for(let n=0;n<6;n++)if(e[n].distanceToPoint(t)<0)return!1;return!0}clone(){return new this.constructor().copy(this)}};function fh(){let i=null,t=!1,e=null,n=null;function s(r,o){e(r,o),n=i.requestAnimationFrame(s)}return{start:function(){t!==!0&&e!==null&&(n=i.requestAnimationFrame(s),t=!0)},stop:function(){i.cancelAnimationFrame(n),t=!1},setAnimationLoop:function(r){e=r},setContext:function(r){i=r}}}function Bd(i){let t=new WeakMap;function e(a,c){let l=a.array,h=a.usage,u=l.byteLength,p=i.createBuffer();i.bindBuffer(c,p),i.bufferData(c,l,h),a.onUploadCallback();let f;if(l instanceof Float32Array)f=i.FLOAT;else if(l instanceof Uint16Array)a.isFloat16BufferAttribute?f=i.HALF_FLOAT:f=i.UNSIGNED_SHORT;else if(l instanceof Int16Array)f=i.SHORT;else if(l instanceof Uint32Array)f=i.UNSIGNED_INT;else if(l instanceof Int32Array)f=i.INT;else if(l instanceof Int8Array)f=i.BYTE;else if(l instanceof Uint8Array)f=i.UNSIGNED_BYTE;else if(l instanceof Uint8ClampedArray)f=i.UNSIGNED_BYTE;else throw new Error("THREE.WebGLAttributes: Unsupported buffer data format: "+l);return{buffer:p,type:f,bytesPerElement:l.BYTES_PER_ELEMENT,version:a.version,size:u}}function n(a,c,l){let h=c.array,u=c.updateRanges;if(i.bindBuffer(l,a),u.length===0)i.bufferSubData(l,0,h);else{u.sort((f,g)=>f.start-g.start);let p=0;for(let f=1;f<u.length;f++){let g=u[p],y=u[f];y.start<=g.start+g.count+1?g.count=Math.max(g.count,y.start+y.count-g.start):(++p,u[p]=y)}u.length=p+1;for(let f=0,g=u.length;f<g;f++){let y=u[f];i.bufferSubData(l,y.start*h.BYTES_PER_ELEMENT,h,y.start,y.count)}c.clearUpdateRanges()}c.onUploadCallback()}function s(a){return a.isInterleavedBufferAttribute&&(a=a.data),t.get(a)}function r(a){a.isInterleavedBufferAttribute&&(a=a.data);let c=t.get(a);c&&(i.deleteBuffer(c.buffer),t.delete(a))}function o(a,c){if(a.isInterleavedBufferAttribute&&(a=a.data),a.isGLBufferAttribute){let h=t.get(a);(!h||h.version<a.version)&&t.set(a,{buffer:a.buffer,type:a.type,bytesPerElement:a.elementSize,version:a.version});return}let l=t.get(a);if(l===void 0)t.set(a,e(a,c));else if(l.version<a.version){if(l.size!==a.array.byteLength)throw new Error("THREE.WebGLAttributes: The size of the buffer attribute's array buffer does not match the original size. Resizing buffer attributes is not supported.");n(l.buffer,a,c),l.version=a.version}}return{get:s,remove:r,update:o}}var wn=class i extends Ge{constructor(t=1,e=1,n=1,s=1){super(),this.type="PlaneGeometry",this.parameters={width:t,height:e,widthSegments:n,heightSegments:s};let r=t/2,o=e/2,a=Math.floor(n),c=Math.floor(s),l=a+1,h=c+1,u=t/a,p=e/c,f=[],g=[],y=[],m=[];for(let d=0;d<h;d++){let x=d*p-o;for(let _=0;_<l;_++){let v=_*u-r;g.push(v,-x,0),y.push(0,0,1),m.push(_/a),m.push(1-d/c)}}for(let d=0;d<c;d++)for(let x=0;x<a;x++){let _=x+l*d,v=x+l*(d+1),C=x+1+l*(d+1),w=x+1+l*d;f.push(_,v,w),f.push(v,C,w)}this.setIndex(f),this.setAttribute("position",new Ce(g,3)),this.setAttribute("normal",new Ce(y,3)),this.setAttribute("uv",new Ce(m,2))}copy(t){return super.copy(t),this.parameters=Object.assign({},t.parameters),this}static fromJSON(t){return new i(t.width,t.height,t.widthSegments,t.heightSegments)}},zd=`#ifdef USE_ALPHAHASH
	if ( diffuseColor.a < getAlphaHashThreshold( vPosition ) ) discard;
#endif`,kd=`#ifdef USE_ALPHAHASH
	const float ALPHA_HASH_SCALE = 0.05;
	float hash2D( vec2 value ) {
		return fract( 1.0e4 * sin( 17.0 * value.x + 0.1 * value.y ) * ( 0.1 + abs( sin( 13.0 * value.y + value.x ) ) ) );
	}
	float hash3D( vec3 value ) {
		return hash2D( vec2( hash2D( value.xy ), value.z ) );
	}
	float getAlphaHashThreshold( vec3 position ) {
		float maxDeriv = max(
			length( dFdx( position.xyz ) ),
			length( dFdy( position.xyz ) )
		);
		float pixScale = 1.0 / ( ALPHA_HASH_SCALE * maxDeriv );
		vec2 pixScales = vec2(
			exp2( floor( log2( pixScale ) ) ),
			exp2( ceil( log2( pixScale ) ) )
		);
		vec2 alpha = vec2(
			hash3D( floor( pixScales.x * position.xyz ) ),
			hash3D( floor( pixScales.y * position.xyz ) )
		);
		float lerpFactor = fract( log2( pixScale ) );
		float x = ( 1.0 - lerpFactor ) * alpha.x + lerpFactor * alpha.y;
		float a = min( lerpFactor, 1.0 - lerpFactor );
		vec3 cases = vec3(
			x * x / ( 2.0 * a * ( 1.0 - a ) ),
			( x - 0.5 * a ) / ( 1.0 - a ),
			1.0 - ( ( 1.0 - x ) * ( 1.0 - x ) / ( 2.0 * a * ( 1.0 - a ) ) )
		);
		float threshold = ( x < ( 1.0 - a ) )
			? ( ( x < a ) ? cases.x : cases.y )
			: cases.z;
		return clamp( threshold , 1.0e-6, 1.0 );
	}
#endif`,Hd=`#ifdef USE_ALPHAMAP
	diffuseColor.a *= texture2D( alphaMap, vAlphaMapUv ).g;
#endif`,Vd=`#ifdef USE_ALPHAMAP
	uniform sampler2D alphaMap;
#endif`,Gd=`#ifdef USE_ALPHATEST
	#ifdef ALPHA_TO_COVERAGE
	diffuseColor.a = smoothstep( alphaTest, alphaTest + fwidth( diffuseColor.a ), diffuseColor.a );
	if ( diffuseColor.a == 0.0 ) discard;
	#else
	if ( diffuseColor.a < alphaTest ) discard;
	#endif
#endif`,Wd=`#ifdef USE_ALPHATEST
	uniform float alphaTest;
#endif`,Xd=`#ifdef USE_AOMAP
	float ambientOcclusion = ( texture2D( aoMap, vAoMapUv ).r - 1.0 ) * aoMapIntensity + 1.0;
	reflectedLight.indirectDiffuse *= ambientOcclusion;
	#if defined( USE_CLEARCOAT ) 
		clearcoatSpecularIndirect *= ambientOcclusion;
	#endif
	#if defined( USE_SHEEN ) 
		sheenSpecularIndirect *= ambientOcclusion;
	#endif
	#if defined( USE_ENVMAP ) && defined( STANDARD )
		float dotNV = saturate( dot( geometryNormal, geometryViewDir ) );
		reflectedLight.indirectSpecular *= computeSpecularOcclusion( dotNV, ambientOcclusion, material.roughness );
	#endif
#endif`,Yd=`#ifdef USE_AOMAP
	uniform sampler2D aoMap;
	uniform float aoMapIntensity;
#endif`,qd=`#ifdef USE_BATCHING
	#if ! defined( GL_ANGLE_multi_draw )
	#define gl_DrawID _gl_DrawID
	uniform int _gl_DrawID;
	#endif
	uniform highp sampler2D batchingTexture;
	uniform highp usampler2D batchingIdTexture;
	mat4 getBatchingMatrix( const in float i ) {
		int size = textureSize( batchingTexture, 0 ).x;
		int j = int( i ) * 4;
		int x = j % size;
		int y = j / size;
		vec4 v1 = texelFetch( batchingTexture, ivec2( x, y ), 0 );
		vec4 v2 = texelFetch( batchingTexture, ivec2( x + 1, y ), 0 );
		vec4 v3 = texelFetch( batchingTexture, ivec2( x + 2, y ), 0 );
		vec4 v4 = texelFetch( batchingTexture, ivec2( x + 3, y ), 0 );
		return mat4( v1, v2, v3, v4 );
	}
	float getIndirectIndex( const in int i ) {
		int size = textureSize( batchingIdTexture, 0 ).x;
		int x = i % size;
		int y = i / size;
		return float( texelFetch( batchingIdTexture, ivec2( x, y ), 0 ).r );
	}
#endif
#ifdef USE_BATCHING_COLOR
	uniform sampler2D batchingColorTexture;
	vec3 getBatchingColor( const in float i ) {
		int size = textureSize( batchingColorTexture, 0 ).x;
		int j = int( i );
		int x = j % size;
		int y = j / size;
		return texelFetch( batchingColorTexture, ivec2( x, y ), 0 ).rgb;
	}
#endif`,$d=`#ifdef USE_BATCHING
	mat4 batchingMatrix = getBatchingMatrix( getIndirectIndex( gl_DrawID ) );
#endif`,Zd=`vec3 transformed = vec3( position );
#ifdef USE_ALPHAHASH
	vPosition = vec3( position );
#endif`,Kd=`vec3 objectNormal = vec3( normal );
#ifdef USE_TANGENT
	vec3 objectTangent = vec3( tangent.xyz );
#endif`,Jd=`float G_BlinnPhong_Implicit( ) {
	return 0.25;
}
float D_BlinnPhong( const in float shininess, const in float dotNH ) {
	return RECIPROCAL_PI * ( shininess * 0.5 + 1.0 ) * pow( dotNH, shininess );
}
vec3 BRDF_BlinnPhong( const in vec3 lightDir, const in vec3 viewDir, const in vec3 normal, const in vec3 specularColor, const in float shininess ) {
	vec3 halfDir = normalize( lightDir + viewDir );
	float dotNH = saturate( dot( normal, halfDir ) );
	float dotVH = saturate( dot( viewDir, halfDir ) );
	vec3 F = F_Schlick( specularColor, 1.0, dotVH );
	float G = G_BlinnPhong_Implicit( );
	float D = D_BlinnPhong( shininess, dotNH );
	return F * ( G * D );
} // validated`,jd=`#ifdef USE_IRIDESCENCE
	const mat3 XYZ_TO_REC709 = mat3(
		 3.2404542, -0.9692660,  0.0556434,
		-1.5371385,  1.8760108, -0.2040259,
		-0.4985314,  0.0415560,  1.0572252
	);
	vec3 Fresnel0ToIor( vec3 fresnel0 ) {
		vec3 sqrtF0 = sqrt( fresnel0 );
		return ( vec3( 1.0 ) + sqrtF0 ) / ( vec3( 1.0 ) - sqrtF0 );
	}
	vec3 IorToFresnel0( vec3 transmittedIor, float incidentIor ) {
		return pow2( ( transmittedIor - vec3( incidentIor ) ) / ( transmittedIor + vec3( incidentIor ) ) );
	}
	float IorToFresnel0( float transmittedIor, float incidentIor ) {
		return pow2( ( transmittedIor - incidentIor ) / ( transmittedIor + incidentIor ));
	}
	vec3 evalSensitivity( float OPD, vec3 shift ) {
		float phase = 2.0 * PI * OPD * 1.0e-9;
		vec3 val = vec3( 5.4856e-13, 4.4201e-13, 5.2481e-13 );
		vec3 pos = vec3( 1.6810e+06, 1.7953e+06, 2.2084e+06 );
		vec3 var = vec3( 4.3278e+09, 9.3046e+09, 6.6121e+09 );
		vec3 xyz = val * sqrt( 2.0 * PI * var ) * cos( pos * phase + shift ) * exp( - pow2( phase ) * var );
		xyz.x += 9.7470e-14 * sqrt( 2.0 * PI * 4.5282e+09 ) * cos( 2.2399e+06 * phase + shift[ 0 ] ) * exp( - 4.5282e+09 * pow2( phase ) );
		xyz /= 1.0685e-7;
		vec3 rgb = XYZ_TO_REC709 * xyz;
		return rgb;
	}
	vec3 evalIridescence( float outsideIOR, float eta2, float cosTheta1, float thinFilmThickness, vec3 baseF0 ) {
		vec3 I;
		float iridescenceIOR = mix( outsideIOR, eta2, smoothstep( 0.0, 0.03, thinFilmThickness ) );
		float sinTheta2Sq = pow2( outsideIOR / iridescenceIOR ) * ( 1.0 - pow2( cosTheta1 ) );
		float cosTheta2Sq = 1.0 - sinTheta2Sq;
		if ( cosTheta2Sq < 0.0 ) {
			return vec3( 1.0 );
		}
		float cosTheta2 = sqrt( cosTheta2Sq );
		float R0 = IorToFresnel0( iridescenceIOR, outsideIOR );
		float R12 = F_Schlick( R0, 1.0, cosTheta1 );
		float T121 = 1.0 - R12;
		float phi12 = 0.0;
		if ( iridescenceIOR < outsideIOR ) phi12 = PI;
		float phi21 = PI - phi12;
		vec3 baseIOR = Fresnel0ToIor( clamp( baseF0, 0.0, 0.9999 ) );		vec3 R1 = IorToFresnel0( baseIOR, iridescenceIOR );
		vec3 R23 = F_Schlick( R1, 1.0, cosTheta2 );
		vec3 phi23 = vec3( 0.0 );
		if ( baseIOR[ 0 ] < iridescenceIOR ) phi23[ 0 ] = PI;
		if ( baseIOR[ 1 ] < iridescenceIOR ) phi23[ 1 ] = PI;
		if ( baseIOR[ 2 ] < iridescenceIOR ) phi23[ 2 ] = PI;
		float OPD = 2.0 * iridescenceIOR * thinFilmThickness * cosTheta2;
		vec3 phi = vec3( phi21 ) + phi23;
		vec3 R123 = clamp( R12 * R23, 1e-5, 0.9999 );
		vec3 r123 = sqrt( R123 );
		vec3 Rs = pow2( T121 ) * R23 / ( vec3( 1.0 ) - R123 );
		vec3 C0 = R12 + Rs;
		I = C0;
		vec3 Cm = Rs - T121;
		for ( int m = 1; m <= 2; ++ m ) {
			Cm *= r123;
			vec3 Sm = 2.0 * evalSensitivity( float( m ) * OPD, float( m ) * phi );
			I += Cm * Sm;
		}
		return max( I, vec3( 0.0 ) );
	}
#endif`,Qd=`#ifdef USE_BUMPMAP
	uniform sampler2D bumpMap;
	uniform float bumpScale;
	vec2 dHdxy_fwd() {
		vec2 dSTdx = dFdx( vBumpMapUv );
		vec2 dSTdy = dFdy( vBumpMapUv );
		float Hll = bumpScale * texture2D( bumpMap, vBumpMapUv ).x;
		float dBx = bumpScale * texture2D( bumpMap, vBumpMapUv + dSTdx ).x - Hll;
		float dBy = bumpScale * texture2D( bumpMap, vBumpMapUv + dSTdy ).x - Hll;
		return vec2( dBx, dBy );
	}
	vec3 perturbNormalArb( vec3 surf_pos, vec3 surf_norm, vec2 dHdxy, float faceDirection ) {
		vec3 vSigmaX = normalize( dFdx( surf_pos.xyz ) );
		vec3 vSigmaY = normalize( dFdy( surf_pos.xyz ) );
		vec3 vN = surf_norm;
		vec3 R1 = cross( vSigmaY, vN );
		vec3 R2 = cross( vN, vSigmaX );
		float fDet = dot( vSigmaX, R1 ) * faceDirection;
		vec3 vGrad = sign( fDet ) * ( dHdxy.x * R1 + dHdxy.y * R2 );
		return normalize( abs( fDet ) * surf_norm - vGrad );
	}
#endif`,tf=`#if NUM_CLIPPING_PLANES > 0
	vec4 plane;
	#ifdef ALPHA_TO_COVERAGE
		float distanceToPlane, distanceGradient;
		float clipOpacity = 1.0;
		#pragma unroll_loop_start
		for ( int i = 0; i < UNION_CLIPPING_PLANES; i ++ ) {
			plane = clippingPlanes[ i ];
			distanceToPlane = - dot( vClipPosition, plane.xyz ) + plane.w;
			distanceGradient = fwidth( distanceToPlane ) / 2.0;
			clipOpacity *= smoothstep( - distanceGradient, distanceGradient, distanceToPlane );
			if ( clipOpacity == 0.0 ) discard;
		}
		#pragma unroll_loop_end
		#if UNION_CLIPPING_PLANES < NUM_CLIPPING_PLANES
			float unionClipOpacity = 1.0;
			#pragma unroll_loop_start
			for ( int i = UNION_CLIPPING_PLANES; i < NUM_CLIPPING_PLANES; i ++ ) {
				plane = clippingPlanes[ i ];
				distanceToPlane = - dot( vClipPosition, plane.xyz ) + plane.w;
				distanceGradient = fwidth( distanceToPlane ) / 2.0;
				unionClipOpacity *= 1.0 - smoothstep( - distanceGradient, distanceGradient, distanceToPlane );
			}
			#pragma unroll_loop_end
			clipOpacity *= 1.0 - unionClipOpacity;
		#endif
		diffuseColor.a *= clipOpacity;
		if ( diffuseColor.a == 0.0 ) discard;
	#else
		#pragma unroll_loop_start
		for ( int i = 0; i < UNION_CLIPPING_PLANES; i ++ ) {
			plane = clippingPlanes[ i ];
			if ( dot( vClipPosition, plane.xyz ) > plane.w ) discard;
		}
		#pragma unroll_loop_end
		#if UNION_CLIPPING_PLANES < NUM_CLIPPING_PLANES
			bool clipped = true;
			#pragma unroll_loop_start
			for ( int i = UNION_CLIPPING_PLANES; i < NUM_CLIPPING_PLANES; i ++ ) {
				plane = clippingPlanes[ i ];
				clipped = ( dot( vClipPosition, plane.xyz ) > plane.w ) && clipped;
			}
			#pragma unroll_loop_end
			if ( clipped ) discard;
		#endif
	#endif
#endif`,ef=`#if NUM_CLIPPING_PLANES > 0
	varying vec3 vClipPosition;
	uniform vec4 clippingPlanes[ NUM_CLIPPING_PLANES ];
#endif`,nf=`#if NUM_CLIPPING_PLANES > 0
	varying vec3 vClipPosition;
#endif`,sf=`#if NUM_CLIPPING_PLANES > 0
	vClipPosition = - mvPosition.xyz;
#endif`,rf=`#if defined( USE_COLOR_ALPHA )
	diffuseColor *= vColor;
#elif defined( USE_COLOR )
	diffuseColor.rgb *= vColor;
#endif`,of=`#if defined( USE_COLOR_ALPHA )
	varying vec4 vColor;
#elif defined( USE_COLOR )
	varying vec3 vColor;
#endif`,af=`#if defined( USE_COLOR_ALPHA )
	varying vec4 vColor;
#elif defined( USE_COLOR ) || defined( USE_INSTANCING_COLOR ) || defined( USE_BATCHING_COLOR )
	varying vec3 vColor;
#endif`,cf=`#if defined( USE_COLOR_ALPHA )
	vColor = vec4( 1.0 );
#elif defined( USE_COLOR ) || defined( USE_INSTANCING_COLOR ) || defined( USE_BATCHING_COLOR )
	vColor = vec3( 1.0 );
#endif
#ifdef USE_COLOR
	vColor *= color;
#endif
#ifdef USE_INSTANCING_COLOR
	vColor.xyz *= instanceColor.xyz;
#endif
#ifdef USE_BATCHING_COLOR
	vec3 batchingColor = getBatchingColor( getIndirectIndex( gl_DrawID ) );
	vColor.xyz *= batchingColor.xyz;
#endif`,lf=`#define PI 3.141592653589793
#define PI2 6.283185307179586
#define PI_HALF 1.5707963267948966
#define RECIPROCAL_PI 0.3183098861837907
#define RECIPROCAL_PI2 0.15915494309189535
#define EPSILON 1e-6
#ifndef saturate
#define saturate( a ) clamp( a, 0.0, 1.0 )
#endif
#define whiteComplement( a ) ( 1.0 - saturate( a ) )
float pow2( const in float x ) { return x*x; }
vec3 pow2( const in vec3 x ) { return x*x; }
float pow3( const in float x ) { return x*x*x; }
float pow4( const in float x ) { float x2 = x*x; return x2*x2; }
float max3( const in vec3 v ) { return max( max( v.x, v.y ), v.z ); }
float average( const in vec3 v ) { return dot( v, vec3( 0.3333333 ) ); }
highp float rand( const in vec2 uv ) {
	const highp float a = 12.9898, b = 78.233, c = 43758.5453;
	highp float dt = dot( uv.xy, vec2( a,b ) ), sn = mod( dt, PI );
	return fract( sin( sn ) * c );
}
#ifdef HIGH_PRECISION
	float precisionSafeLength( vec3 v ) { return length( v ); }
#else
	float precisionSafeLength( vec3 v ) {
		float maxComponent = max3( abs( v ) );
		return length( v / maxComponent ) * maxComponent;
	}
#endif
struct IncidentLight {
	vec3 color;
	vec3 direction;
	bool visible;
};
struct ReflectedLight {
	vec3 directDiffuse;
	vec3 directSpecular;
	vec3 indirectDiffuse;
	vec3 indirectSpecular;
};
#ifdef USE_ALPHAHASH
	varying vec3 vPosition;
#endif
vec3 transformDirection( in vec3 dir, in mat4 matrix ) {
	return normalize( ( matrix * vec4( dir, 0.0 ) ).xyz );
}
vec3 inverseTransformDirection( in vec3 dir, in mat4 matrix ) {
	return normalize( ( vec4( dir, 0.0 ) * matrix ).xyz );
}
mat3 transposeMat3( const in mat3 m ) {
	mat3 tmp;
	tmp[ 0 ] = vec3( m[ 0 ].x, m[ 1 ].x, m[ 2 ].x );
	tmp[ 1 ] = vec3( m[ 0 ].y, m[ 1 ].y, m[ 2 ].y );
	tmp[ 2 ] = vec3( m[ 0 ].z, m[ 1 ].z, m[ 2 ].z );
	return tmp;
}
bool isPerspectiveMatrix( mat4 m ) {
	return m[ 2 ][ 3 ] == - 1.0;
}
vec2 equirectUv( in vec3 dir ) {
	float u = atan( dir.z, dir.x ) * RECIPROCAL_PI2 + 0.5;
	float v = asin( clamp( dir.y, - 1.0, 1.0 ) ) * RECIPROCAL_PI + 0.5;
	return vec2( u, v );
}
vec3 BRDF_Lambert( const in vec3 diffuseColor ) {
	return RECIPROCAL_PI * diffuseColor;
}
vec3 F_Schlick( const in vec3 f0, const in float f90, const in float dotVH ) {
	float fresnel = exp2( ( - 5.55473 * dotVH - 6.98316 ) * dotVH );
	return f0 * ( 1.0 - fresnel ) + ( f90 * fresnel );
}
float F_Schlick( const in float f0, const in float f90, const in float dotVH ) {
	float fresnel = exp2( ( - 5.55473 * dotVH - 6.98316 ) * dotVH );
	return f0 * ( 1.0 - fresnel ) + ( f90 * fresnel );
} // validated`,hf=`#ifdef ENVMAP_TYPE_CUBE_UV
	#define cubeUV_minMipLevel 4.0
	#define cubeUV_minTileSize 16.0
	float getFace( vec3 direction ) {
		vec3 absDirection = abs( direction );
		float face = - 1.0;
		if ( absDirection.x > absDirection.z ) {
			if ( absDirection.x > absDirection.y )
				face = direction.x > 0.0 ? 0.0 : 3.0;
			else
				face = direction.y > 0.0 ? 1.0 : 4.0;
		} else {
			if ( absDirection.z > absDirection.y )
				face = direction.z > 0.0 ? 2.0 : 5.0;
			else
				face = direction.y > 0.0 ? 1.0 : 4.0;
		}
		return face;
	}
	vec2 getUV( vec3 direction, float face ) {
		vec2 uv;
		if ( face == 0.0 ) {
			uv = vec2( direction.z, direction.y ) / abs( direction.x );
		} else if ( face == 1.0 ) {
			uv = vec2( - direction.x, - direction.z ) / abs( direction.y );
		} else if ( face == 2.0 ) {
			uv = vec2( - direction.x, direction.y ) / abs( direction.z );
		} else if ( face == 3.0 ) {
			uv = vec2( - direction.z, direction.y ) / abs( direction.x );
		} else if ( face == 4.0 ) {
			uv = vec2( - direction.x, direction.z ) / abs( direction.y );
		} else {
			uv = vec2( direction.x, direction.y ) / abs( direction.z );
		}
		return 0.5 * ( uv + 1.0 );
	}
	vec3 bilinearCubeUV( sampler2D envMap, vec3 direction, float mipInt ) {
		float face = getFace( direction );
		float filterInt = max( cubeUV_minMipLevel - mipInt, 0.0 );
		mipInt = max( mipInt, cubeUV_minMipLevel );
		float faceSize = exp2( mipInt );
		highp vec2 uv = getUV( direction, face ) * ( faceSize - 2.0 ) + 1.0;
		if ( face > 2.0 ) {
			uv.y += faceSize;
			face -= 3.0;
		}
		uv.x += face * faceSize;
		uv.x += filterInt * 3.0 * cubeUV_minTileSize;
		uv.y += 4.0 * ( exp2( CUBEUV_MAX_MIP ) - faceSize );
		uv.x *= CUBEUV_TEXEL_WIDTH;
		uv.y *= CUBEUV_TEXEL_HEIGHT;
		#ifdef texture2DGradEXT
			return texture2DGradEXT( envMap, uv, vec2( 0.0 ), vec2( 0.0 ) ).rgb;
		#else
			return texture2D( envMap, uv ).rgb;
		#endif
	}
	#define cubeUV_r0 1.0
	#define cubeUV_m0 - 2.0
	#define cubeUV_r1 0.8
	#define cubeUV_m1 - 1.0
	#define cubeUV_r4 0.4
	#define cubeUV_m4 2.0
	#define cubeUV_r5 0.305
	#define cubeUV_m5 3.0
	#define cubeUV_r6 0.21
	#define cubeUV_m6 4.0
	float roughnessToMip( float roughness ) {
		float mip = 0.0;
		if ( roughness >= cubeUV_r1 ) {
			mip = ( cubeUV_r0 - roughness ) * ( cubeUV_m1 - cubeUV_m0 ) / ( cubeUV_r0 - cubeUV_r1 ) + cubeUV_m0;
		} else if ( roughness >= cubeUV_r4 ) {
			mip = ( cubeUV_r1 - roughness ) * ( cubeUV_m4 - cubeUV_m1 ) / ( cubeUV_r1 - cubeUV_r4 ) + cubeUV_m1;
		} else if ( roughness >= cubeUV_r5 ) {
			mip = ( cubeUV_r4 - roughness ) * ( cubeUV_m5 - cubeUV_m4 ) / ( cubeUV_r4 - cubeUV_r5 ) + cubeUV_m4;
		} else if ( roughness >= cubeUV_r6 ) {
			mip = ( cubeUV_r5 - roughness ) * ( cubeUV_m6 - cubeUV_m5 ) / ( cubeUV_r5 - cubeUV_r6 ) + cubeUV_m5;
		} else {
			mip = - 2.0 * log2( 1.16 * roughness );		}
		return mip;
	}
	vec4 textureCubeUV( sampler2D envMap, vec3 sampleDir, float roughness ) {
		float mip = clamp( roughnessToMip( roughness ), cubeUV_m0, CUBEUV_MAX_MIP );
		float mipF = fract( mip );
		float mipInt = floor( mip );
		vec3 color0 = bilinearCubeUV( envMap, sampleDir, mipInt );
		if ( mipF == 0.0 ) {
			return vec4( color0, 1.0 );
		} else {
			vec3 color1 = bilinearCubeUV( envMap, sampleDir, mipInt + 1.0 );
			return vec4( mix( color0, color1, mipF ), 1.0 );
		}
	}
#endif`,uf=`vec3 transformedNormal = objectNormal;
#ifdef USE_TANGENT
	vec3 transformedTangent = objectTangent;
#endif
#ifdef USE_BATCHING
	mat3 bm = mat3( batchingMatrix );
	transformedNormal /= vec3( dot( bm[ 0 ], bm[ 0 ] ), dot( bm[ 1 ], bm[ 1 ] ), dot( bm[ 2 ], bm[ 2 ] ) );
	transformedNormal = bm * transformedNormal;
	#ifdef USE_TANGENT
		transformedTangent = bm * transformedTangent;
	#endif
#endif
#ifdef USE_INSTANCING
	mat3 im = mat3( instanceMatrix );
	transformedNormal /= vec3( dot( im[ 0 ], im[ 0 ] ), dot( im[ 1 ], im[ 1 ] ), dot( im[ 2 ], im[ 2 ] ) );
	transformedNormal = im * transformedNormal;
	#ifdef USE_TANGENT
		transformedTangent = im * transformedTangent;
	#endif
#endif
transformedNormal = normalMatrix * transformedNormal;
#ifdef FLIP_SIDED
	transformedNormal = - transformedNormal;
#endif
#ifdef USE_TANGENT
	transformedTangent = ( modelViewMatrix * vec4( transformedTangent, 0.0 ) ).xyz;
	#ifdef FLIP_SIDED
		transformedTangent = - transformedTangent;
	#endif
#endif`,df=`#ifdef USE_DISPLACEMENTMAP
	uniform sampler2D displacementMap;
	uniform float displacementScale;
	uniform float displacementBias;
#endif`,ff=`#ifdef USE_DISPLACEMENTMAP
	transformed += normalize( objectNormal ) * ( texture2D( displacementMap, vDisplacementMapUv ).x * displacementScale + displacementBias );
#endif`,pf=`#ifdef USE_EMISSIVEMAP
	vec4 emissiveColor = texture2D( emissiveMap, vEmissiveMapUv );
	#ifdef DECODE_VIDEO_TEXTURE_EMISSIVE
		emissiveColor = sRGBTransferEOTF( emissiveColor );
	#endif
	totalEmissiveRadiance *= emissiveColor.rgb;
#endif`,mf=`#ifdef USE_EMISSIVEMAP
	uniform sampler2D emissiveMap;
#endif`,gf="gl_FragColor = linearToOutputTexel( gl_FragColor );",_f=`vec4 LinearTransferOETF( in vec4 value ) {
	return value;
}
vec4 sRGBTransferEOTF( in vec4 value ) {
	return vec4( mix( pow( value.rgb * 0.9478672986 + vec3( 0.0521327014 ), vec3( 2.4 ) ), value.rgb * 0.0773993808, vec3( lessThanEqual( value.rgb, vec3( 0.04045 ) ) ) ), value.a );
}
vec4 sRGBTransferOETF( in vec4 value ) {
	return vec4( mix( pow( value.rgb, vec3( 0.41666 ) ) * 1.055 - vec3( 0.055 ), value.rgb * 12.92, vec3( lessThanEqual( value.rgb, vec3( 0.0031308 ) ) ) ), value.a );
}`,xf=`#ifdef USE_ENVMAP
	#ifdef ENV_WORLDPOS
		vec3 cameraToFrag;
		if ( isOrthographic ) {
			cameraToFrag = normalize( vec3( - viewMatrix[ 0 ][ 2 ], - viewMatrix[ 1 ][ 2 ], - viewMatrix[ 2 ][ 2 ] ) );
		} else {
			cameraToFrag = normalize( vWorldPosition - cameraPosition );
		}
		vec3 worldNormal = inverseTransformDirection( normal, viewMatrix );
		#ifdef ENVMAP_MODE_REFLECTION
			vec3 reflectVec = reflect( cameraToFrag, worldNormal );
		#else
			vec3 reflectVec = refract( cameraToFrag, worldNormal, refractionRatio );
		#endif
	#else
		vec3 reflectVec = vReflect;
	#endif
	#ifdef ENVMAP_TYPE_CUBE
		vec4 envColor = textureCube( envMap, envMapRotation * vec3( flipEnvMap * reflectVec.x, reflectVec.yz ) );
	#else
		vec4 envColor = vec4( 0.0 );
	#endif
	#ifdef ENVMAP_BLENDING_MULTIPLY
		outgoingLight = mix( outgoingLight, outgoingLight * envColor.xyz, specularStrength * reflectivity );
	#elif defined( ENVMAP_BLENDING_MIX )
		outgoingLight = mix( outgoingLight, envColor.xyz, specularStrength * reflectivity );
	#elif defined( ENVMAP_BLENDING_ADD )
		outgoingLight += envColor.xyz * specularStrength * reflectivity;
	#endif
#endif`,yf=`#ifdef USE_ENVMAP
	uniform float envMapIntensity;
	uniform float flipEnvMap;
	uniform mat3 envMapRotation;
	#ifdef ENVMAP_TYPE_CUBE
		uniform samplerCube envMap;
	#else
		uniform sampler2D envMap;
	#endif
	
#endif`,vf=`#ifdef USE_ENVMAP
	uniform float reflectivity;
	#if defined( USE_BUMPMAP ) || defined( USE_NORMALMAP ) || defined( PHONG ) || defined( LAMBERT )
		#define ENV_WORLDPOS
	#endif
	#ifdef ENV_WORLDPOS
		varying vec3 vWorldPosition;
		uniform float refractionRatio;
	#else
		varying vec3 vReflect;
	#endif
#endif`,Mf=`#ifdef USE_ENVMAP
	#if defined( USE_BUMPMAP ) || defined( USE_NORMALMAP ) || defined( PHONG ) || defined( LAMBERT )
		#define ENV_WORLDPOS
	#endif
	#ifdef ENV_WORLDPOS
		
		varying vec3 vWorldPosition;
	#else
		varying vec3 vReflect;
		uniform float refractionRatio;
	#endif
#endif`,Sf=`#ifdef USE_ENVMAP
	#ifdef ENV_WORLDPOS
		vWorldPosition = worldPosition.xyz;
	#else
		vec3 cameraToVertex;
		if ( isOrthographic ) {
			cameraToVertex = normalize( vec3( - viewMatrix[ 0 ][ 2 ], - viewMatrix[ 1 ][ 2 ], - viewMatrix[ 2 ][ 2 ] ) );
		} else {
			cameraToVertex = normalize( worldPosition.xyz - cameraPosition );
		}
		vec3 worldNormal = inverseTransformDirection( transformedNormal, viewMatrix );
		#ifdef ENVMAP_MODE_REFLECTION
			vReflect = reflect( cameraToVertex, worldNormal );
		#else
			vReflect = refract( cameraToVertex, worldNormal, refractionRatio );
		#endif
	#endif
#endif`,bf=`#ifdef USE_FOG
	vFogDepth = - mvPosition.z;
#endif`,Ef=`#ifdef USE_FOG
	varying float vFogDepth;
#endif`,Tf=`#ifdef USE_FOG
	#ifdef FOG_EXP2
		float fogFactor = 1.0 - exp( - fogDensity * fogDensity * vFogDepth * vFogDepth );
	#else
		float fogFactor = smoothstep( fogNear, fogFar, vFogDepth );
	#endif
	gl_FragColor.rgb = mix( gl_FragColor.rgb, fogColor, fogFactor );
#endif`,wf=`#ifdef USE_FOG
	uniform vec3 fogColor;
	varying float vFogDepth;
	#ifdef FOG_EXP2
		uniform float fogDensity;
	#else
		uniform float fogNear;
		uniform float fogFar;
	#endif
#endif`,Af=`#ifdef USE_GRADIENTMAP
	uniform sampler2D gradientMap;
#endif
vec3 getGradientIrradiance( vec3 normal, vec3 lightDirection ) {
	float dotNL = dot( normal, lightDirection );
	vec2 coord = vec2( dotNL * 0.5 + 0.5, 0.0 );
	#ifdef USE_GRADIENTMAP
		return vec3( texture2D( gradientMap, coord ).r );
	#else
		vec2 fw = fwidth( coord ) * 0.5;
		return mix( vec3( 0.7 ), vec3( 1.0 ), smoothstep( 0.7 - fw.x, 0.7 + fw.x, coord.x ) );
	#endif
}`,Cf=`#ifdef USE_LIGHTMAP
	uniform sampler2D lightMap;
	uniform float lightMapIntensity;
#endif`,Rf=`LambertMaterial material;
material.diffuseColor = diffuseColor.rgb;
material.specularStrength = specularStrength;`,Pf=`varying vec3 vViewPosition;
struct LambertMaterial {
	vec3 diffuseColor;
	float specularStrength;
};
void RE_Direct_Lambert( const in IncidentLight directLight, const in vec3 geometryPosition, const in vec3 geometryNormal, const in vec3 geometryViewDir, const in vec3 geometryClearcoatNormal, const in LambertMaterial material, inout ReflectedLight reflectedLight ) {
	float dotNL = saturate( dot( geometryNormal, directLight.direction ) );
	vec3 irradiance = dotNL * directLight.color;
	reflectedLight.directDiffuse += irradiance * BRDF_Lambert( material.diffuseColor );
}
void RE_IndirectDiffuse_Lambert( const in vec3 irradiance, const in vec3 geometryPosition, const in vec3 geometryNormal, const in vec3 geometryViewDir, const in vec3 geometryClearcoatNormal, const in LambertMaterial material, inout ReflectedLight reflectedLight ) {
	reflectedLight.indirectDiffuse += irradiance * BRDF_Lambert( material.diffuseColor );
}
#define RE_Direct				RE_Direct_Lambert
#define RE_IndirectDiffuse		RE_IndirectDiffuse_Lambert`,If=`uniform bool receiveShadow;
uniform vec3 ambientLightColor;
#if defined( USE_LIGHT_PROBES )
	uniform vec3 lightProbe[ 9 ];
#endif
vec3 shGetIrradianceAt( in vec3 normal, in vec3 shCoefficients[ 9 ] ) {
	float x = normal.x, y = normal.y, z = normal.z;
	vec3 result = shCoefficients[ 0 ] * 0.886227;
	result += shCoefficients[ 1 ] * 2.0 * 0.511664 * y;
	result += shCoefficients[ 2 ] * 2.0 * 0.511664 * z;
	result += shCoefficients[ 3 ] * 2.0 * 0.511664 * x;
	result += shCoefficients[ 4 ] * 2.0 * 0.429043 * x * y;
	result += shCoefficients[ 5 ] * 2.0 * 0.429043 * y * z;
	result += shCoefficients[ 6 ] * ( 0.743125 * z * z - 0.247708 );
	result += shCoefficients[ 7 ] * 2.0 * 0.429043 * x * z;
	result += shCoefficients[ 8 ] * 0.429043 * ( x * x - y * y );
	return result;
}
vec3 getLightProbeIrradiance( const in vec3 lightProbe[ 9 ], const in vec3 normal ) {
	vec3 worldNormal = inverseTransformDirection( normal, viewMatrix );
	vec3 irradiance = shGetIrradianceAt( worldNormal, lightProbe );
	return irradiance;
}
vec3 getAmbientLightIrradiance( const in vec3 ambientLightColor ) {
	vec3 irradiance = ambientLightColor;
	return irradiance;
}
float getDistanceAttenuation( const in float lightDistance, const in float cutoffDistance, const in float decayExponent ) {
	float distanceFalloff = 1.0 / max( pow( lightDistance, decayExponent ), 0.01 );
	if ( cutoffDistance > 0.0 ) {
		distanceFalloff *= pow2( saturate( 1.0 - pow4( lightDistance / cutoffDistance ) ) );
	}
	return distanceFalloff;
}
float getSpotAttenuation( const in float coneCosine, const in float penumbraCosine, const in float angleCosine ) {
	return smoothstep( coneCosine, penumbraCosine, angleCosine );
}
#if NUM_DIR_LIGHTS > 0
	struct DirectionalLight {
		vec3 direction;
		vec3 color;
	};
	uniform DirectionalLight directionalLights[ NUM_DIR_LIGHTS ];
	void getDirectionalLightInfo( const in DirectionalLight directionalLight, out IncidentLight light ) {
		light.color = directionalLight.color;
		light.direction = directionalLight.direction;
		light.visible = true;
	}
#endif
#if NUM_POINT_LIGHTS > 0
	struct PointLight {
		vec3 position;
		vec3 color;
		float distance;
		float decay;
	};
	uniform PointLight pointLights[ NUM_POINT_LIGHTS ];
	void getPointLightInfo( const in PointLight pointLight, const in vec3 geometryPosition, out IncidentLight light ) {
		vec3 lVector = pointLight.position - geometryPosition;
		light.direction = normalize( lVector );
		float lightDistance = length( lVector );
		light.color = pointLight.color;
		light.color *= getDistanceAttenuation( lightDistance, pointLight.distance, pointLight.decay );
		light.visible = ( light.color != vec3( 0.0 ) );
	}
#endif
#if NUM_SPOT_LIGHTS > 0
	struct SpotLight {
		vec3 position;
		vec3 direction;
		vec3 color;
		float distance;
		float decay;
		float coneCos;
		float penumbraCos;
	};
	uniform SpotLight spotLights[ NUM_SPOT_LIGHTS ];
	void getSpotLightInfo( const in SpotLight spotLight, const in vec3 geometryPosition, out IncidentLight light ) {
		vec3 lVector = spotLight.position - geometryPosition;
		light.direction = normalize( lVector );
		float angleCos = dot( light.direction, spotLight.direction );
		float spotAttenuation = getSpotAttenuation( spotLight.coneCos, spotLight.penumbraCos, angleCos );
		if ( spotAttenuation > 0.0 ) {
			float lightDistance = length( lVector );
			light.color = spotLight.color * spotAttenuation;
			light.color *= getDistanceAttenuation( lightDistance, spotLight.distance, spotLight.decay );
			light.visible = ( light.color != vec3( 0.0 ) );
		} else {
			light.color = vec3( 0.0 );
			light.visible = false;
		}
	}
#endif
#if NUM_RECT_AREA_LIGHTS > 0
	struct RectAreaLight {
		vec3 color;
		vec3 position;
		vec3 halfWidth;
		vec3 halfHeight;
	};
	uniform sampler2D ltc_1;	uniform sampler2D ltc_2;
	uniform RectAreaLight rectAreaLights[ NUM_RECT_AREA_LIGHTS ];
#endif
#if NUM_HEMI_LIGHTS > 0
	struct HemisphereLight {
		vec3 direction;
		vec3 skyColor;
		vec3 groundColor;
	};
	uniform HemisphereLight hemisphereLights[ NUM_HEMI_LIGHTS ];
	vec3 getHemisphereLightIrradiance( const in HemisphereLight hemiLight, const in vec3 normal ) {
		float dotNL = dot( normal, hemiLight.direction );
		float hemiDiffuseWeight = 0.5 * dotNL + 0.5;
		vec3 irradiance = mix( hemiLight.groundColor, hemiLight.skyColor, hemiDiffuseWeight );
		return irradiance;
	}
#endif`,Lf=`#ifdef USE_ENVMAP
	vec3 getIBLIrradiance( const in vec3 normal ) {
		#ifdef ENVMAP_TYPE_CUBE_UV
			vec3 worldNormal = inverseTransformDirection( normal, viewMatrix );
			vec4 envMapColor = textureCubeUV( envMap, envMapRotation * worldNormal, 1.0 );
			return PI * envMapColor.rgb * envMapIntensity;
		#else
			return vec3( 0.0 );
		#endif
	}
	vec3 getIBLRadiance( const in vec3 viewDir, const in vec3 normal, const in float roughness ) {
		#ifdef ENVMAP_TYPE_CUBE_UV
			vec3 reflectVec = reflect( - viewDir, normal );
			reflectVec = normalize( mix( reflectVec, normal, roughness * roughness) );
			reflectVec = inverseTransformDirection( reflectVec, viewMatrix );
			vec4 envMapColor = textureCubeUV( envMap, envMapRotation * reflectVec, roughness );
			return envMapColor.rgb * envMapIntensity;
		#else
			return vec3( 0.0 );
		#endif
	}
	#ifdef USE_ANISOTROPY
		vec3 getIBLAnisotropyRadiance( const in vec3 viewDir, const in vec3 normal, const in float roughness, const in vec3 bitangent, const in float anisotropy ) {
			#ifdef ENVMAP_TYPE_CUBE_UV
				vec3 bentNormal = cross( bitangent, viewDir );
				bentNormal = normalize( cross( bentNormal, bitangent ) );
				bentNormal = normalize( mix( bentNormal, normal, pow2( pow2( 1.0 - anisotropy * ( 1.0 - roughness ) ) ) ) );
				return getIBLRadiance( viewDir, bentNormal, roughness );
			#else
				return vec3( 0.0 );
			#endif
		}
	#endif
#endif`,Df=`ToonMaterial material;
material.diffuseColor = diffuseColor.rgb;`,Uf=`varying vec3 vViewPosition;
struct ToonMaterial {
	vec3 diffuseColor;
};
void RE_Direct_Toon( const in IncidentLight directLight, const in vec3 geometryPosition, const in vec3 geometryNormal, const in vec3 geometryViewDir, const in vec3 geometryClearcoatNormal, const in ToonMaterial material, inout ReflectedLight reflectedLight ) {
	vec3 irradiance = getGradientIrradiance( geometryNormal, directLight.direction ) * directLight.color;
	reflectedLight.directDiffuse += irradiance * BRDF_Lambert( material.diffuseColor );
}
void RE_IndirectDiffuse_Toon( const in vec3 irradiance, const in vec3 geometryPosition, const in vec3 geometryNormal, const in vec3 geometryViewDir, const in vec3 geometryClearcoatNormal, const in ToonMaterial material, inout ReflectedLight reflectedLight ) {
	reflectedLight.indirectDiffuse += irradiance * BRDF_Lambert( material.diffuseColor );
}
#define RE_Direct				RE_Direct_Toon
#define RE_IndirectDiffuse		RE_IndirectDiffuse_Toon`,Nf=`BlinnPhongMaterial material;
material.diffuseColor = diffuseColor.rgb;
material.specularColor = specular;
material.specularShininess = shininess;
material.specularStrength = specularStrength;`,Ff=`varying vec3 vViewPosition;
struct BlinnPhongMaterial {
	vec3 diffuseColor;
	vec3 specularColor;
	float specularShininess;
	float specularStrength;
};
void RE_Direct_BlinnPhong( const in IncidentLight directLight, const in vec3 geometryPosition, const in vec3 geometryNormal, const in vec3 geometryViewDir, const in vec3 geometryClearcoatNormal, const in BlinnPhongMaterial material, inout ReflectedLight reflectedLight ) {
	float dotNL = saturate( dot( geometryNormal, directLight.direction ) );
	vec3 irradiance = dotNL * directLight.color;
	reflectedLight.directDiffuse += irradiance * BRDF_Lambert( material.diffuseColor );
	reflectedLight.directSpecular += irradiance * BRDF_BlinnPhong( directLight.direction, geometryViewDir, geometryNormal, material.specularColor, material.specularShininess ) * material.specularStrength;
}
void RE_IndirectDiffuse_BlinnPhong( const in vec3 irradiance, const in vec3 geometryPosition, const in vec3 geometryNormal, const in vec3 geometryViewDir, const in vec3 geometryClearcoatNormal, const in BlinnPhongMaterial material, inout ReflectedLight reflectedLight ) {
	reflectedLight.indirectDiffuse += irradiance * BRDF_Lambert( material.diffuseColor );
}
#define RE_Direct				RE_Direct_BlinnPhong
#define RE_IndirectDiffuse		RE_IndirectDiffuse_BlinnPhong`,Of=`PhysicalMaterial material;
material.diffuseColor = diffuseColor.rgb * ( 1.0 - metalnessFactor );
vec3 dxy = max( abs( dFdx( nonPerturbedNormal ) ), abs( dFdy( nonPerturbedNormal ) ) );
float geometryRoughness = max( max( dxy.x, dxy.y ), dxy.z );
material.roughness = max( roughnessFactor, 0.0525 );material.roughness += geometryRoughness;
material.roughness = min( material.roughness, 1.0 );
#ifdef IOR
	material.ior = ior;
	#ifdef USE_SPECULAR
		float specularIntensityFactor = specularIntensity;
		vec3 specularColorFactor = specularColor;
		#ifdef USE_SPECULAR_COLORMAP
			specularColorFactor *= texture2D( specularColorMap, vSpecularColorMapUv ).rgb;
		#endif
		#ifdef USE_SPECULAR_INTENSITYMAP
			specularIntensityFactor *= texture2D( specularIntensityMap, vSpecularIntensityMapUv ).a;
		#endif
		material.specularF90 = mix( specularIntensityFactor, 1.0, metalnessFactor );
	#else
		float specularIntensityFactor = 1.0;
		vec3 specularColorFactor = vec3( 1.0 );
		material.specularF90 = 1.0;
	#endif
	material.specularColor = mix( min( pow2( ( material.ior - 1.0 ) / ( material.ior + 1.0 ) ) * specularColorFactor, vec3( 1.0 ) ) * specularIntensityFactor, diffuseColor.rgb, metalnessFactor );
#else
	material.specularColor = mix( vec3( 0.04 ), diffuseColor.rgb, metalnessFactor );
	material.specularF90 = 1.0;
#endif
#ifdef USE_CLEARCOAT
	material.clearcoat = clearcoat;
	material.clearcoatRoughness = clearcoatRoughness;
	material.clearcoatF0 = vec3( 0.04 );
	material.clearcoatF90 = 1.0;
	#ifdef USE_CLEARCOATMAP
		material.clearcoat *= texture2D( clearcoatMap, vClearcoatMapUv ).x;
	#endif
	#ifdef USE_CLEARCOAT_ROUGHNESSMAP
		material.clearcoatRoughness *= texture2D( clearcoatRoughnessMap, vClearcoatRoughnessMapUv ).y;
	#endif
	material.clearcoat = saturate( material.clearcoat );	material.clearcoatRoughness = max( material.clearcoatRoughness, 0.0525 );
	material.clearcoatRoughness += geometryRoughness;
	material.clearcoatRoughness = min( material.clearcoatRoughness, 1.0 );
#endif
#ifdef USE_DISPERSION
	material.dispersion = dispersion;
#endif
#ifdef USE_IRIDESCENCE
	material.iridescence = iridescence;
	material.iridescenceIOR = iridescenceIOR;
	#ifdef USE_IRIDESCENCEMAP
		material.iridescence *= texture2D( iridescenceMap, vIridescenceMapUv ).r;
	#endif
	#ifdef USE_IRIDESCENCE_THICKNESSMAP
		material.iridescenceThickness = (iridescenceThicknessMaximum - iridescenceThicknessMinimum) * texture2D( iridescenceThicknessMap, vIridescenceThicknessMapUv ).g + iridescenceThicknessMinimum;
	#else
		material.iridescenceThickness = iridescenceThicknessMaximum;
	#endif
#endif
#ifdef USE_SHEEN
	material.sheenColor = sheenColor;
	#ifdef USE_SHEEN_COLORMAP
		material.sheenColor *= texture2D( sheenColorMap, vSheenColorMapUv ).rgb;
	#endif
	material.sheenRoughness = clamp( sheenRoughness, 0.07, 1.0 );
	#ifdef USE_SHEEN_ROUGHNESSMAP
		material.sheenRoughness *= texture2D( sheenRoughnessMap, vSheenRoughnessMapUv ).a;
	#endif
#endif
#ifdef USE_ANISOTROPY
	#ifdef USE_ANISOTROPYMAP
		mat2 anisotropyMat = mat2( anisotropyVector.x, anisotropyVector.y, - anisotropyVector.y, anisotropyVector.x );
		vec3 anisotropyPolar = texture2D( anisotropyMap, vAnisotropyMapUv ).rgb;
		vec2 anisotropyV = anisotropyMat * normalize( 2.0 * anisotropyPolar.rg - vec2( 1.0 ) ) * anisotropyPolar.b;
	#else
		vec2 anisotropyV = anisotropyVector;
	#endif
	material.anisotropy = length( anisotropyV );
	if( material.anisotropy == 0.0 ) {
		anisotropyV = vec2( 1.0, 0.0 );
	} else {
		anisotropyV /= material.anisotropy;
		material.anisotropy = saturate( material.anisotropy );
	}
	material.alphaT = mix( pow2( material.roughness ), 1.0, pow2( material.anisotropy ) );
	material.anisotropyT = tbn[ 0 ] * anisotropyV.x + tbn[ 1 ] * anisotropyV.y;
	material.anisotropyB = tbn[ 1 ] * anisotropyV.x - tbn[ 0 ] * anisotropyV.y;
#endif`,Bf=`struct PhysicalMaterial {
	vec3 diffuseColor;
	float roughness;
	vec3 specularColor;
	float specularF90;
	float dispersion;
	#ifdef USE_CLEARCOAT
		float clearcoat;
		float clearcoatRoughness;
		vec3 clearcoatF0;
		float clearcoatF90;
	#endif
	#ifdef USE_IRIDESCENCE
		float iridescence;
		float iridescenceIOR;
		float iridescenceThickness;
		vec3 iridescenceFresnel;
		vec3 iridescenceF0;
	#endif
	#ifdef USE_SHEEN
		vec3 sheenColor;
		float sheenRoughness;
	#endif
	#ifdef IOR
		float ior;
	#endif
	#ifdef USE_TRANSMISSION
		float transmission;
		float transmissionAlpha;
		float thickness;
		float attenuationDistance;
		vec3 attenuationColor;
	#endif
	#ifdef USE_ANISOTROPY
		float anisotropy;
		float alphaT;
		vec3 anisotropyT;
		vec3 anisotropyB;
	#endif
};
vec3 clearcoatSpecularDirect = vec3( 0.0 );
vec3 clearcoatSpecularIndirect = vec3( 0.0 );
vec3 sheenSpecularDirect = vec3( 0.0 );
vec3 sheenSpecularIndirect = vec3(0.0 );
vec3 Schlick_to_F0( const in vec3 f, const in float f90, const in float dotVH ) {
    float x = clamp( 1.0 - dotVH, 0.0, 1.0 );
    float x2 = x * x;
    float x5 = clamp( x * x2 * x2, 0.0, 0.9999 );
    return ( f - vec3( f90 ) * x5 ) / ( 1.0 - x5 );
}
float V_GGX_SmithCorrelated( const in float alpha, const in float dotNL, const in float dotNV ) {
	float a2 = pow2( alpha );
	float gv = dotNL * sqrt( a2 + ( 1.0 - a2 ) * pow2( dotNV ) );
	float gl = dotNV * sqrt( a2 + ( 1.0 - a2 ) * pow2( dotNL ) );
	return 0.5 / max( gv + gl, EPSILON );
}
float D_GGX( const in float alpha, const in float dotNH ) {
	float a2 = pow2( alpha );
	float denom = pow2( dotNH ) * ( a2 - 1.0 ) + 1.0;
	return RECIPROCAL_PI * a2 / pow2( denom );
}
#ifdef USE_ANISOTROPY
	float V_GGX_SmithCorrelated_Anisotropic( const in float alphaT, const in float alphaB, const in float dotTV, const in float dotBV, const in float dotTL, const in float dotBL, const in float dotNV, const in float dotNL ) {
		float gv = dotNL * length( vec3( alphaT * dotTV, alphaB * dotBV, dotNV ) );
		float gl = dotNV * length( vec3( alphaT * dotTL, alphaB * dotBL, dotNL ) );
		float v = 0.5 / ( gv + gl );
		return saturate(v);
	}
	float D_GGX_Anisotropic( const in float alphaT, const in float alphaB, const in float dotNH, const in float dotTH, const in float dotBH ) {
		float a2 = alphaT * alphaB;
		highp vec3 v = vec3( alphaB * dotTH, alphaT * dotBH, a2 * dotNH );
		highp float v2 = dot( v, v );
		float w2 = a2 / v2;
		return RECIPROCAL_PI * a2 * pow2 ( w2 );
	}
#endif
#ifdef USE_CLEARCOAT
	vec3 BRDF_GGX_Clearcoat( const in vec3 lightDir, const in vec3 viewDir, const in vec3 normal, const in PhysicalMaterial material) {
		vec3 f0 = material.clearcoatF0;
		float f90 = material.clearcoatF90;
		float roughness = material.clearcoatRoughness;
		float alpha = pow2( roughness );
		vec3 halfDir = normalize( lightDir + viewDir );
		float dotNL = saturate( dot( normal, lightDir ) );
		float dotNV = saturate( dot( normal, viewDir ) );
		float dotNH = saturate( dot( normal, halfDir ) );
		float dotVH = saturate( dot( viewDir, halfDir ) );
		vec3 F = F_Schlick( f0, f90, dotVH );
		float V = V_GGX_SmithCorrelated( alpha, dotNL, dotNV );
		float D = D_GGX( alpha, dotNH );
		return F * ( V * D );
	}
#endif
vec3 BRDF_GGX( const in vec3 lightDir, const in vec3 viewDir, const in vec3 normal, const in PhysicalMaterial material ) {
	vec3 f0 = material.specularColor;
	float f90 = material.specularF90;
	float roughness = material.roughness;
	float alpha = pow2( roughness );
	vec3 halfDir = normalize( lightDir + viewDir );
	float dotNL = saturate( dot( normal, lightDir ) );
	float dotNV = saturate( dot( normal, viewDir ) );
	float dotNH = saturate( dot( normal, halfDir ) );
	float dotVH = saturate( dot( viewDir, halfDir ) );
	vec3 F = F_Schlick( f0, f90, dotVH );
	#ifdef USE_IRIDESCENCE
		F = mix( F, material.iridescenceFresnel, material.iridescence );
	#endif
	#ifdef USE_ANISOTROPY
		float dotTL = dot( material.anisotropyT, lightDir );
		float dotTV = dot( material.anisotropyT, viewDir );
		float dotTH = dot( material.anisotropyT, halfDir );
		float dotBL = dot( material.anisotropyB, lightDir );
		float dotBV = dot( material.anisotropyB, viewDir );
		float dotBH = dot( material.anisotropyB, halfDir );
		float V = V_GGX_SmithCorrelated_Anisotropic( material.alphaT, alpha, dotTV, dotBV, dotTL, dotBL, dotNV, dotNL );
		float D = D_GGX_Anisotropic( material.alphaT, alpha, dotNH, dotTH, dotBH );
	#else
		float V = V_GGX_SmithCorrelated( alpha, dotNL, dotNV );
		float D = D_GGX( alpha, dotNH );
	#endif
	return F * ( V * D );
}
vec2 LTC_Uv( const in vec3 N, const in vec3 V, const in float roughness ) {
	const float LUT_SIZE = 64.0;
	const float LUT_SCALE = ( LUT_SIZE - 1.0 ) / LUT_SIZE;
	const float LUT_BIAS = 0.5 / LUT_SIZE;
	float dotNV = saturate( dot( N, V ) );
	vec2 uv = vec2( roughness, sqrt( 1.0 - dotNV ) );
	uv = uv * LUT_SCALE + LUT_BIAS;
	return uv;
}
float LTC_ClippedSphereFormFactor( const in vec3 f ) {
	float l = length( f );
	return max( ( l * l + f.z ) / ( l + 1.0 ), 0.0 );
}
vec3 LTC_EdgeVectorFormFactor( const in vec3 v1, const in vec3 v2 ) {
	float x = dot( v1, v2 );
	float y = abs( x );
	float a = 0.8543985 + ( 0.4965155 + 0.0145206 * y ) * y;
	float b = 3.4175940 + ( 4.1616724 + y ) * y;
	float v = a / b;
	float theta_sintheta = ( x > 0.0 ) ? v : 0.5 * inversesqrt( max( 1.0 - x * x, 1e-7 ) ) - v;
	return cross( v1, v2 ) * theta_sintheta;
}
vec3 LTC_Evaluate( const in vec3 N, const in vec3 V, const in vec3 P, const in mat3 mInv, const in vec3 rectCoords[ 4 ] ) {
	vec3 v1 = rectCoords[ 1 ] - rectCoords[ 0 ];
	vec3 v2 = rectCoords[ 3 ] - rectCoords[ 0 ];
	vec3 lightNormal = cross( v1, v2 );
	if( dot( lightNormal, P - rectCoords[ 0 ] ) < 0.0 ) return vec3( 0.0 );
	vec3 T1, T2;
	T1 = normalize( V - N * dot( V, N ) );
	T2 = - cross( N, T1 );
	mat3 mat = mInv * transposeMat3( mat3( T1, T2, N ) );
	vec3 coords[ 4 ];
	coords[ 0 ] = mat * ( rectCoords[ 0 ] - P );
	coords[ 1 ] = mat * ( rectCoords[ 1 ] - P );
	coords[ 2 ] = mat * ( rectCoords[ 2 ] - P );
	coords[ 3 ] = mat * ( rectCoords[ 3 ] - P );
	coords[ 0 ] = normalize( coords[ 0 ] );
	coords[ 1 ] = normalize( coords[ 1 ] );
	coords[ 2 ] = normalize( coords[ 2 ] );
	coords[ 3 ] = normalize( coords[ 3 ] );
	vec3 vectorFormFactor = vec3( 0.0 );
	vectorFormFactor += LTC_EdgeVectorFormFactor( coords[ 0 ], coords[ 1 ] );
	vectorFormFactor += LTC_EdgeVectorFormFactor( coords[ 1 ], coords[ 2 ] );
	vectorFormFactor += LTC_EdgeVectorFormFactor( coords[ 2 ], coords[ 3 ] );
	vectorFormFactor += LTC_EdgeVectorFormFactor( coords[ 3 ], coords[ 0 ] );
	float result = LTC_ClippedSphereFormFactor( vectorFormFactor );
	return vec3( result );
}
#if defined( USE_SHEEN )
float D_Charlie( float roughness, float dotNH ) {
	float alpha = pow2( roughness );
	float invAlpha = 1.0 / alpha;
	float cos2h = dotNH * dotNH;
	float sin2h = max( 1.0 - cos2h, 0.0078125 );
	return ( 2.0 + invAlpha ) * pow( sin2h, invAlpha * 0.5 ) / ( 2.0 * PI );
}
float V_Neubelt( float dotNV, float dotNL ) {
	return saturate( 1.0 / ( 4.0 * ( dotNL + dotNV - dotNL * dotNV ) ) );
}
vec3 BRDF_Sheen( const in vec3 lightDir, const in vec3 viewDir, const in vec3 normal, vec3 sheenColor, const in float sheenRoughness ) {
	vec3 halfDir = normalize( lightDir + viewDir );
	float dotNL = saturate( dot( normal, lightDir ) );
	float dotNV = saturate( dot( normal, viewDir ) );
	float dotNH = saturate( dot( normal, halfDir ) );
	float D = D_Charlie( sheenRoughness, dotNH );
	float V = V_Neubelt( dotNV, dotNL );
	return sheenColor * ( D * V );
}
#endif
float IBLSheenBRDF( const in vec3 normal, const in vec3 viewDir, const in float roughness ) {
	float dotNV = saturate( dot( normal, viewDir ) );
	float r2 = roughness * roughness;
	float a = roughness < 0.25 ? -339.2 * r2 + 161.4 * roughness - 25.9 : -8.48 * r2 + 14.3 * roughness - 9.95;
	float b = roughness < 0.25 ? 44.0 * r2 - 23.7 * roughness + 3.26 : 1.97 * r2 - 3.27 * roughness + 0.72;
	float DG = exp( a * dotNV + b ) + ( roughness < 0.25 ? 0.0 : 0.1 * ( roughness - 0.25 ) );
	return saturate( DG * RECIPROCAL_PI );
}
vec2 DFGApprox( const in vec3 normal, const in vec3 viewDir, const in float roughness ) {
	float dotNV = saturate( dot( normal, viewDir ) );
	const vec4 c0 = vec4( - 1, - 0.0275, - 0.572, 0.022 );
	const vec4 c1 = vec4( 1, 0.0425, 1.04, - 0.04 );
	vec4 r = roughness * c0 + c1;
	float a004 = min( r.x * r.x, exp2( - 9.28 * dotNV ) ) * r.x + r.y;
	vec2 fab = vec2( - 1.04, 1.04 ) * a004 + r.zw;
	return fab;
}
vec3 EnvironmentBRDF( const in vec3 normal, const in vec3 viewDir, const in vec3 specularColor, const in float specularF90, const in float roughness ) {
	vec2 fab = DFGApprox( normal, viewDir, roughness );
	return specularColor * fab.x + specularF90 * fab.y;
}
#ifdef USE_IRIDESCENCE
void computeMultiscatteringIridescence( const in vec3 normal, const in vec3 viewDir, const in vec3 specularColor, const in float specularF90, const in float iridescence, const in vec3 iridescenceF0, const in float roughness, inout vec3 singleScatter, inout vec3 multiScatter ) {
#else
void computeMultiscattering( const in vec3 normal, const in vec3 viewDir, const in vec3 specularColor, const in float specularF90, const in float roughness, inout vec3 singleScatter, inout vec3 multiScatter ) {
#endif
	vec2 fab = DFGApprox( normal, viewDir, roughness );
	#ifdef USE_IRIDESCENCE
		vec3 Fr = mix( specularColor, iridescenceF0, iridescence );
	#else
		vec3 Fr = specularColor;
	#endif
	vec3 FssEss = Fr * fab.x + specularF90 * fab.y;
	float Ess = fab.x + fab.y;
	float Ems = 1.0 - Ess;
	vec3 Favg = Fr + ( 1.0 - Fr ) * 0.047619;	vec3 Fms = FssEss * Favg / ( 1.0 - Ems * Favg );
	singleScatter += FssEss;
	multiScatter += Fms * Ems;
}
#if NUM_RECT_AREA_LIGHTS > 0
	void RE_Direct_RectArea_Physical( const in RectAreaLight rectAreaLight, const in vec3 geometryPosition, const in vec3 geometryNormal, const in vec3 geometryViewDir, const in vec3 geometryClearcoatNormal, const in PhysicalMaterial material, inout ReflectedLight reflectedLight ) {
		vec3 normal = geometryNormal;
		vec3 viewDir = geometryViewDir;
		vec3 position = geometryPosition;
		vec3 lightPos = rectAreaLight.position;
		vec3 halfWidth = rectAreaLight.halfWidth;
		vec3 halfHeight = rectAreaLight.halfHeight;
		vec3 lightColor = rectAreaLight.color;
		float roughness = material.roughness;
		vec3 rectCoords[ 4 ];
		rectCoords[ 0 ] = lightPos + halfWidth - halfHeight;		rectCoords[ 1 ] = lightPos - halfWidth - halfHeight;
		rectCoords[ 2 ] = lightPos - halfWidth + halfHeight;
		rectCoords[ 3 ] = lightPos + halfWidth + halfHeight;
		vec2 uv = LTC_Uv( normal, viewDir, roughness );
		vec4 t1 = texture2D( ltc_1, uv );
		vec4 t2 = texture2D( ltc_2, uv );
		mat3 mInv = mat3(
			vec3( t1.x, 0, t1.y ),
			vec3(    0, 1,    0 ),
			vec3( t1.z, 0, t1.w )
		);
		vec3 fresnel = ( material.specularColor * t2.x + ( vec3( 1.0 ) - material.specularColor ) * t2.y );
		reflectedLight.directSpecular += lightColor * fresnel * LTC_Evaluate( normal, viewDir, position, mInv, rectCoords );
		reflectedLight.directDiffuse += lightColor * material.diffuseColor * LTC_Evaluate( normal, viewDir, position, mat3( 1.0 ), rectCoords );
	}
#endif
void RE_Direct_Physical( const in IncidentLight directLight, const in vec3 geometryPosition, const in vec3 geometryNormal, const in vec3 geometryViewDir, const in vec3 geometryClearcoatNormal, const in PhysicalMaterial material, inout ReflectedLight reflectedLight ) {
	float dotNL = saturate( dot( geometryNormal, directLight.direction ) );
	vec3 irradiance = dotNL * directLight.color;
	#ifdef USE_CLEARCOAT
		float dotNLcc = saturate( dot( geometryClearcoatNormal, directLight.direction ) );
		vec3 ccIrradiance = dotNLcc * directLight.color;
		clearcoatSpecularDirect += ccIrradiance * BRDF_GGX_Clearcoat( directLight.direction, geometryViewDir, geometryClearcoatNormal, material );
	#endif
	#ifdef USE_SHEEN
		sheenSpecularDirect += irradiance * BRDF_Sheen( directLight.direction, geometryViewDir, geometryNormal, material.sheenColor, material.sheenRoughness );
	#endif
	reflectedLight.directSpecular += irradiance * BRDF_GGX( directLight.direction, geometryViewDir, geometryNormal, material );
	reflectedLight.directDiffuse += irradiance * BRDF_Lambert( material.diffuseColor );
}
void RE_IndirectDiffuse_Physical( const in vec3 irradiance, const in vec3 geometryPosition, const in vec3 geometryNormal, const in vec3 geometryViewDir, const in vec3 geometryClearcoatNormal, const in PhysicalMaterial material, inout ReflectedLight reflectedLight ) {
	reflectedLight.indirectDiffuse += irradiance * BRDF_Lambert( material.diffuseColor );
}
void RE_IndirectSpecular_Physical( const in vec3 radiance, const in vec3 irradiance, const in vec3 clearcoatRadiance, const in vec3 geometryPosition, const in vec3 geometryNormal, const in vec3 geometryViewDir, const in vec3 geometryClearcoatNormal, const in PhysicalMaterial material, inout ReflectedLight reflectedLight) {
	#ifdef USE_CLEARCOAT
		clearcoatSpecularIndirect += clearcoatRadiance * EnvironmentBRDF( geometryClearcoatNormal, geometryViewDir, material.clearcoatF0, material.clearcoatF90, material.clearcoatRoughness );
	#endif
	#ifdef USE_SHEEN
		sheenSpecularIndirect += irradiance * material.sheenColor * IBLSheenBRDF( geometryNormal, geometryViewDir, material.sheenRoughness );
	#endif
	vec3 singleScattering = vec3( 0.0 );
	vec3 multiScattering = vec3( 0.0 );
	vec3 cosineWeightedIrradiance = irradiance * RECIPROCAL_PI;
	#ifdef USE_IRIDESCENCE
		computeMultiscatteringIridescence( geometryNormal, geometryViewDir, material.specularColor, material.specularF90, material.iridescence, material.iridescenceFresnel, material.roughness, singleScattering, multiScattering );
	#else
		computeMultiscattering( geometryNormal, geometryViewDir, material.specularColor, material.specularF90, material.roughness, singleScattering, multiScattering );
	#endif
	vec3 totalScattering = singleScattering + multiScattering;
	vec3 diffuse = material.diffuseColor * ( 1.0 - max( max( totalScattering.r, totalScattering.g ), totalScattering.b ) );
	reflectedLight.indirectSpecular += radiance * singleScattering;
	reflectedLight.indirectSpecular += multiScattering * cosineWeightedIrradiance;
	reflectedLight.indirectDiffuse += diffuse * cosineWeightedIrradiance;
}
#define RE_Direct				RE_Direct_Physical
#define RE_Direct_RectArea		RE_Direct_RectArea_Physical
#define RE_IndirectDiffuse		RE_IndirectDiffuse_Physical
#define RE_IndirectSpecular		RE_IndirectSpecular_Physical
float computeSpecularOcclusion( const in float dotNV, const in float ambientOcclusion, const in float roughness ) {
	return saturate( pow( dotNV + ambientOcclusion, exp2( - 16.0 * roughness - 1.0 ) ) - 1.0 + ambientOcclusion );
}`,zf=`
vec3 geometryPosition = - vViewPosition;
vec3 geometryNormal = normal;
vec3 geometryViewDir = ( isOrthographic ) ? vec3( 0, 0, 1 ) : normalize( vViewPosition );
vec3 geometryClearcoatNormal = vec3( 0.0 );
#ifdef USE_CLEARCOAT
	geometryClearcoatNormal = clearcoatNormal;
#endif
#ifdef USE_IRIDESCENCE
	float dotNVi = saturate( dot( normal, geometryViewDir ) );
	if ( material.iridescenceThickness == 0.0 ) {
		material.iridescence = 0.0;
	} else {
		material.iridescence = saturate( material.iridescence );
	}
	if ( material.iridescence > 0.0 ) {
		material.iridescenceFresnel = evalIridescence( 1.0, material.iridescenceIOR, dotNVi, material.iridescenceThickness, material.specularColor );
		material.iridescenceF0 = Schlick_to_F0( material.iridescenceFresnel, 1.0, dotNVi );
	}
#endif
IncidentLight directLight;
#if ( NUM_POINT_LIGHTS > 0 ) && defined( RE_Direct )
	PointLight pointLight;
	#if defined( USE_SHADOWMAP ) && NUM_POINT_LIGHT_SHADOWS > 0
	PointLightShadow pointLightShadow;
	#endif
	#pragma unroll_loop_start
	for ( int i = 0; i < NUM_POINT_LIGHTS; i ++ ) {
		pointLight = pointLights[ i ];
		getPointLightInfo( pointLight, geometryPosition, directLight );
		#if defined( USE_SHADOWMAP ) && ( UNROLLED_LOOP_INDEX < NUM_POINT_LIGHT_SHADOWS )
		pointLightShadow = pointLightShadows[ i ];
		directLight.color *= ( directLight.visible && receiveShadow ) ? getPointShadow( pointShadowMap[ i ], pointLightShadow.shadowMapSize, pointLightShadow.shadowIntensity, pointLightShadow.shadowBias, pointLightShadow.shadowRadius, vPointShadowCoord[ i ], pointLightShadow.shadowCameraNear, pointLightShadow.shadowCameraFar ) : 1.0;
		#endif
		RE_Direct( directLight, geometryPosition, geometryNormal, geometryViewDir, geometryClearcoatNormal, material, reflectedLight );
	}
	#pragma unroll_loop_end
#endif
#if ( NUM_SPOT_LIGHTS > 0 ) && defined( RE_Direct )
	SpotLight spotLight;
	vec4 spotColor;
	vec3 spotLightCoord;
	bool inSpotLightMap;
	#if defined( USE_SHADOWMAP ) && NUM_SPOT_LIGHT_SHADOWS > 0
	SpotLightShadow spotLightShadow;
	#endif
	#pragma unroll_loop_start
	for ( int i = 0; i < NUM_SPOT_LIGHTS; i ++ ) {
		spotLight = spotLights[ i ];
		getSpotLightInfo( spotLight, geometryPosition, directLight );
		#if ( UNROLLED_LOOP_INDEX < NUM_SPOT_LIGHT_SHADOWS_WITH_MAPS )
		#define SPOT_LIGHT_MAP_INDEX UNROLLED_LOOP_INDEX
		#elif ( UNROLLED_LOOP_INDEX < NUM_SPOT_LIGHT_SHADOWS )
		#define SPOT_LIGHT_MAP_INDEX NUM_SPOT_LIGHT_MAPS
		#else
		#define SPOT_LIGHT_MAP_INDEX ( UNROLLED_LOOP_INDEX - NUM_SPOT_LIGHT_SHADOWS + NUM_SPOT_LIGHT_SHADOWS_WITH_MAPS )
		#endif
		#if ( SPOT_LIGHT_MAP_INDEX < NUM_SPOT_LIGHT_MAPS )
			spotLightCoord = vSpotLightCoord[ i ].xyz / vSpotLightCoord[ i ].w;
			inSpotLightMap = all( lessThan( abs( spotLightCoord * 2. - 1. ), vec3( 1.0 ) ) );
			spotColor = texture2D( spotLightMap[ SPOT_LIGHT_MAP_INDEX ], spotLightCoord.xy );
			directLight.color = inSpotLightMap ? directLight.color * spotColor.rgb : directLight.color;
		#endif
		#undef SPOT_LIGHT_MAP_INDEX
		#if defined( USE_SHADOWMAP ) && ( UNROLLED_LOOP_INDEX < NUM_SPOT_LIGHT_SHADOWS )
		spotLightShadow = spotLightShadows[ i ];
		directLight.color *= ( directLight.visible && receiveShadow ) ? getShadow( spotShadowMap[ i ], spotLightShadow.shadowMapSize, spotLightShadow.shadowIntensity, spotLightShadow.shadowBias, spotLightShadow.shadowRadius, vSpotLightCoord[ i ] ) : 1.0;
		#endif
		RE_Direct( directLight, geometryPosition, geometryNormal, geometryViewDir, geometryClearcoatNormal, material, reflectedLight );
	}
	#pragma unroll_loop_end
#endif
#if ( NUM_DIR_LIGHTS > 0 ) && defined( RE_Direct )
	DirectionalLight directionalLight;
	#if defined( USE_SHADOWMAP ) && NUM_DIR_LIGHT_SHADOWS > 0
	DirectionalLightShadow directionalLightShadow;
	#endif
	#pragma unroll_loop_start
	for ( int i = 0; i < NUM_DIR_LIGHTS; i ++ ) {
		directionalLight = directionalLights[ i ];
		getDirectionalLightInfo( directionalLight, directLight );
		#if defined( USE_SHADOWMAP ) && ( UNROLLED_LOOP_INDEX < NUM_DIR_LIGHT_SHADOWS )
		directionalLightShadow = directionalLightShadows[ i ];
		directLight.color *= ( directLight.visible && receiveShadow ) ? getShadow( directionalShadowMap[ i ], directionalLightShadow.shadowMapSize, directionalLightShadow.shadowIntensity, directionalLightShadow.shadowBias, directionalLightShadow.shadowRadius, vDirectionalShadowCoord[ i ] ) : 1.0;
		#endif
		RE_Direct( directLight, geometryPosition, geometryNormal, geometryViewDir, geometryClearcoatNormal, material, reflectedLight );
	}
	#pragma unroll_loop_end
#endif
#if ( NUM_RECT_AREA_LIGHTS > 0 ) && defined( RE_Direct_RectArea )
	RectAreaLight rectAreaLight;
	#pragma unroll_loop_start
	for ( int i = 0; i < NUM_RECT_AREA_LIGHTS; i ++ ) {
		rectAreaLight = rectAreaLights[ i ];
		RE_Direct_RectArea( rectAreaLight, geometryPosition, geometryNormal, geometryViewDir, geometryClearcoatNormal, material, reflectedLight );
	}
	#pragma unroll_loop_end
#endif
#if defined( RE_IndirectDiffuse )
	vec3 iblIrradiance = vec3( 0.0 );
	vec3 irradiance = getAmbientLightIrradiance( ambientLightColor );
	#if defined( USE_LIGHT_PROBES )
		irradiance += getLightProbeIrradiance( lightProbe, geometryNormal );
	#endif
	#if ( NUM_HEMI_LIGHTS > 0 )
		#pragma unroll_loop_start
		for ( int i = 0; i < NUM_HEMI_LIGHTS; i ++ ) {
			irradiance += getHemisphereLightIrradiance( hemisphereLights[ i ], geometryNormal );
		}
		#pragma unroll_loop_end
	#endif
#endif
#if defined( RE_IndirectSpecular )
	vec3 radiance = vec3( 0.0 );
	vec3 clearcoatRadiance = vec3( 0.0 );
#endif`,kf=`#if defined( RE_IndirectDiffuse )
	#ifdef USE_LIGHTMAP
		vec4 lightMapTexel = texture2D( lightMap, vLightMapUv );
		vec3 lightMapIrradiance = lightMapTexel.rgb * lightMapIntensity;
		irradiance += lightMapIrradiance;
	#endif
	#if defined( USE_ENVMAP ) && defined( STANDARD ) && defined( ENVMAP_TYPE_CUBE_UV )
		iblIrradiance += getIBLIrradiance( geometryNormal );
	#endif
#endif
#if defined( USE_ENVMAP ) && defined( RE_IndirectSpecular )
	#ifdef USE_ANISOTROPY
		radiance += getIBLAnisotropyRadiance( geometryViewDir, geometryNormal, material.roughness, material.anisotropyB, material.anisotropy );
	#else
		radiance += getIBLRadiance( geometryViewDir, geometryNormal, material.roughness );
	#endif
	#ifdef USE_CLEARCOAT
		clearcoatRadiance += getIBLRadiance( geometryViewDir, geometryClearcoatNormal, material.clearcoatRoughness );
	#endif
#endif`,Hf=`#if defined( RE_IndirectDiffuse )
	RE_IndirectDiffuse( irradiance, geometryPosition, geometryNormal, geometryViewDir, geometryClearcoatNormal, material, reflectedLight );
#endif
#if defined( RE_IndirectSpecular )
	RE_IndirectSpecular( radiance, iblIrradiance, clearcoatRadiance, geometryPosition, geometryNormal, geometryViewDir, geometryClearcoatNormal, material, reflectedLight );
#endif`,Vf=`#if defined( USE_LOGDEPTHBUF )
	gl_FragDepth = vIsPerspective == 0.0 ? gl_FragCoord.z : log2( vFragDepth ) * logDepthBufFC * 0.5;
#endif`,Gf=`#if defined( USE_LOGDEPTHBUF )
	uniform float logDepthBufFC;
	varying float vFragDepth;
	varying float vIsPerspective;
#endif`,Wf=`#ifdef USE_LOGDEPTHBUF
	varying float vFragDepth;
	varying float vIsPerspective;
#endif`,Xf=`#ifdef USE_LOGDEPTHBUF
	vFragDepth = 1.0 + gl_Position.w;
	vIsPerspective = float( isPerspectiveMatrix( projectionMatrix ) );
#endif`,Yf=`#ifdef USE_MAP
	vec4 sampledDiffuseColor = texture2D( map, vMapUv );
	#ifdef DECODE_VIDEO_TEXTURE
		sampledDiffuseColor = sRGBTransferEOTF( sampledDiffuseColor );
	#endif
	diffuseColor *= sampledDiffuseColor;
#endif`,qf=`#ifdef USE_MAP
	uniform sampler2D map;
#endif`,$f=`#if defined( USE_MAP ) || defined( USE_ALPHAMAP )
	#if defined( USE_POINTS_UV )
		vec2 uv = vUv;
	#else
		vec2 uv = ( uvTransform * vec3( gl_PointCoord.x, 1.0 - gl_PointCoord.y, 1 ) ).xy;
	#endif
#endif
#ifdef USE_MAP
	diffuseColor *= texture2D( map, uv );
#endif
#ifdef USE_ALPHAMAP
	diffuseColor.a *= texture2D( alphaMap, uv ).g;
#endif`,Zf=`#if defined( USE_POINTS_UV )
	varying vec2 vUv;
#else
	#if defined( USE_MAP ) || defined( USE_ALPHAMAP )
		uniform mat3 uvTransform;
	#endif
#endif
#ifdef USE_MAP
	uniform sampler2D map;
#endif
#ifdef USE_ALPHAMAP
	uniform sampler2D alphaMap;
#endif`,Kf=`float metalnessFactor = metalness;
#ifdef USE_METALNESSMAP
	vec4 texelMetalness = texture2D( metalnessMap, vMetalnessMapUv );
	metalnessFactor *= texelMetalness.b;
#endif`,Jf=`#ifdef USE_METALNESSMAP
	uniform sampler2D metalnessMap;
#endif`,jf=`#ifdef USE_INSTANCING_MORPH
	float morphTargetInfluences[ MORPHTARGETS_COUNT ];
	float morphTargetBaseInfluence = texelFetch( morphTexture, ivec2( 0, gl_InstanceID ), 0 ).r;
	for ( int i = 0; i < MORPHTARGETS_COUNT; i ++ ) {
		morphTargetInfluences[i] =  texelFetch( morphTexture, ivec2( i + 1, gl_InstanceID ), 0 ).r;
	}
#endif`,Qf=`#if defined( USE_MORPHCOLORS )
	vColor *= morphTargetBaseInfluence;
	for ( int i = 0; i < MORPHTARGETS_COUNT; i ++ ) {
		#if defined( USE_COLOR_ALPHA )
			if ( morphTargetInfluences[ i ] != 0.0 ) vColor += getMorph( gl_VertexID, i, 2 ) * morphTargetInfluences[ i ];
		#elif defined( USE_COLOR )
			if ( morphTargetInfluences[ i ] != 0.0 ) vColor += getMorph( gl_VertexID, i, 2 ).rgb * morphTargetInfluences[ i ];
		#endif
	}
#endif`,tp=`#ifdef USE_MORPHNORMALS
	objectNormal *= morphTargetBaseInfluence;
	for ( int i = 0; i < MORPHTARGETS_COUNT; i ++ ) {
		if ( morphTargetInfluences[ i ] != 0.0 ) objectNormal += getMorph( gl_VertexID, i, 1 ).xyz * morphTargetInfluences[ i ];
	}
#endif`,ep=`#ifdef USE_MORPHTARGETS
	#ifndef USE_INSTANCING_MORPH
		uniform float morphTargetBaseInfluence;
		uniform float morphTargetInfluences[ MORPHTARGETS_COUNT ];
	#endif
	uniform sampler2DArray morphTargetsTexture;
	uniform ivec2 morphTargetsTextureSize;
	vec4 getMorph( const in int vertexIndex, const in int morphTargetIndex, const in int offset ) {
		int texelIndex = vertexIndex * MORPHTARGETS_TEXTURE_STRIDE + offset;
		int y = texelIndex / morphTargetsTextureSize.x;
		int x = texelIndex - y * morphTargetsTextureSize.x;
		ivec3 morphUV = ivec3( x, y, morphTargetIndex );
		return texelFetch( morphTargetsTexture, morphUV, 0 );
	}
#endif`,np=`#ifdef USE_MORPHTARGETS
	transformed *= morphTargetBaseInfluence;
	for ( int i = 0; i < MORPHTARGETS_COUNT; i ++ ) {
		if ( morphTargetInfluences[ i ] != 0.0 ) transformed += getMorph( gl_VertexID, i, 0 ).xyz * morphTargetInfluences[ i ];
	}
#endif`,ip=`float faceDirection = gl_FrontFacing ? 1.0 : - 1.0;
#ifdef FLAT_SHADED
	vec3 fdx = dFdx( vViewPosition );
	vec3 fdy = dFdy( vViewPosition );
	vec3 normal = normalize( cross( fdx, fdy ) );
#else
	vec3 normal = normalize( vNormal );
	#ifdef DOUBLE_SIDED
		normal *= faceDirection;
	#endif
#endif
#if defined( USE_NORMALMAP_TANGENTSPACE ) || defined( USE_CLEARCOAT_NORMALMAP ) || defined( USE_ANISOTROPY )
	#ifdef USE_TANGENT
		mat3 tbn = mat3( normalize( vTangent ), normalize( vBitangent ), normal );
	#else
		mat3 tbn = getTangentFrame( - vViewPosition, normal,
		#if defined( USE_NORMALMAP )
			vNormalMapUv
		#elif defined( USE_CLEARCOAT_NORMALMAP )
			vClearcoatNormalMapUv
		#else
			vUv
		#endif
		);
	#endif
	#if defined( DOUBLE_SIDED ) && ! defined( FLAT_SHADED )
		tbn[0] *= faceDirection;
		tbn[1] *= faceDirection;
	#endif
#endif
#ifdef USE_CLEARCOAT_NORMALMAP
	#ifdef USE_TANGENT
		mat3 tbn2 = mat3( normalize( vTangent ), normalize( vBitangent ), normal );
	#else
		mat3 tbn2 = getTangentFrame( - vViewPosition, normal, vClearcoatNormalMapUv );
	#endif
	#if defined( DOUBLE_SIDED ) && ! defined( FLAT_SHADED )
		tbn2[0] *= faceDirection;
		tbn2[1] *= faceDirection;
	#endif
#endif
vec3 nonPerturbedNormal = normal;`,sp=`#ifdef USE_NORMALMAP_OBJECTSPACE
	normal = texture2D( normalMap, vNormalMapUv ).xyz * 2.0 - 1.0;
	#ifdef FLIP_SIDED
		normal = - normal;
	#endif
	#ifdef DOUBLE_SIDED
		normal = normal * faceDirection;
	#endif
	normal = normalize( normalMatrix * normal );
#elif defined( USE_NORMALMAP_TANGENTSPACE )
	vec3 mapN = texture2D( normalMap, vNormalMapUv ).xyz * 2.0 - 1.0;
	mapN.xy *= normalScale;
	normal = normalize( tbn * mapN );
#elif defined( USE_BUMPMAP )
	normal = perturbNormalArb( - vViewPosition, normal, dHdxy_fwd(), faceDirection );
#endif`,rp=`#ifndef FLAT_SHADED
	varying vec3 vNormal;
	#ifdef USE_TANGENT
		varying vec3 vTangent;
		varying vec3 vBitangent;
	#endif
#endif`,op=`#ifndef FLAT_SHADED
	varying vec3 vNormal;
	#ifdef USE_TANGENT
		varying vec3 vTangent;
		varying vec3 vBitangent;
	#endif
#endif`,ap=`#ifndef FLAT_SHADED
	vNormal = normalize( transformedNormal );
	#ifdef USE_TANGENT
		vTangent = normalize( transformedTangent );
		vBitangent = normalize( cross( vNormal, vTangent ) * tangent.w );
	#endif
#endif`,cp=`#ifdef USE_NORMALMAP
	uniform sampler2D normalMap;
	uniform vec2 normalScale;
#endif
#ifdef USE_NORMALMAP_OBJECTSPACE
	uniform mat3 normalMatrix;
#endif
#if ! defined ( USE_TANGENT ) && ( defined ( USE_NORMALMAP_TANGENTSPACE ) || defined ( USE_CLEARCOAT_NORMALMAP ) || defined( USE_ANISOTROPY ) )
	mat3 getTangentFrame( vec3 eye_pos, vec3 surf_norm, vec2 uv ) {
		vec3 q0 = dFdx( eye_pos.xyz );
		vec3 q1 = dFdy( eye_pos.xyz );
		vec2 st0 = dFdx( uv.st );
		vec2 st1 = dFdy( uv.st );
		vec3 N = surf_norm;
		vec3 q1perp = cross( q1, N );
		vec3 q0perp = cross( N, q0 );
		vec3 T = q1perp * st0.x + q0perp * st1.x;
		vec3 B = q1perp * st0.y + q0perp * st1.y;
		float det = max( dot( T, T ), dot( B, B ) );
		float scale = ( det == 0.0 ) ? 0.0 : inversesqrt( det );
		return mat3( T * scale, B * scale, N );
	}
#endif`,lp=`#ifdef USE_CLEARCOAT
	vec3 clearcoatNormal = nonPerturbedNormal;
#endif`,hp=`#ifdef USE_CLEARCOAT_NORMALMAP
	vec3 clearcoatMapN = texture2D( clearcoatNormalMap, vClearcoatNormalMapUv ).xyz * 2.0 - 1.0;
	clearcoatMapN.xy *= clearcoatNormalScale;
	clearcoatNormal = normalize( tbn2 * clearcoatMapN );
#endif`,up=`#ifdef USE_CLEARCOATMAP
	uniform sampler2D clearcoatMap;
#endif
#ifdef USE_CLEARCOAT_NORMALMAP
	uniform sampler2D clearcoatNormalMap;
	uniform vec2 clearcoatNormalScale;
#endif
#ifdef USE_CLEARCOAT_ROUGHNESSMAP
	uniform sampler2D clearcoatRoughnessMap;
#endif`,dp=`#ifdef USE_IRIDESCENCEMAP
	uniform sampler2D iridescenceMap;
#endif
#ifdef USE_IRIDESCENCE_THICKNESSMAP
	uniform sampler2D iridescenceThicknessMap;
#endif`,fp=`#ifdef OPAQUE
diffuseColor.a = 1.0;
#endif
#ifdef USE_TRANSMISSION
diffuseColor.a *= material.transmissionAlpha;
#endif
gl_FragColor = vec4( outgoingLight, diffuseColor.a );`,pp=`vec3 packNormalToRGB( const in vec3 normal ) {
	return normalize( normal ) * 0.5 + 0.5;
}
vec3 unpackRGBToNormal( const in vec3 rgb ) {
	return 2.0 * rgb.xyz - 1.0;
}
const float PackUpscale = 256. / 255.;const float UnpackDownscale = 255. / 256.;const float ShiftRight8 = 1. / 256.;
const float Inv255 = 1. / 255.;
const vec4 PackFactors = vec4( 1.0, 256.0, 256.0 * 256.0, 256.0 * 256.0 * 256.0 );
const vec2 UnpackFactors2 = vec2( UnpackDownscale, 1.0 / PackFactors.g );
const vec3 UnpackFactors3 = vec3( UnpackDownscale / PackFactors.rg, 1.0 / PackFactors.b );
const vec4 UnpackFactors4 = vec4( UnpackDownscale / PackFactors.rgb, 1.0 / PackFactors.a );
vec4 packDepthToRGBA( const in float v ) {
	if( v <= 0.0 )
		return vec4( 0., 0., 0., 0. );
	if( v >= 1.0 )
		return vec4( 1., 1., 1., 1. );
	float vuf;
	float af = modf( v * PackFactors.a, vuf );
	float bf = modf( vuf * ShiftRight8, vuf );
	float gf = modf( vuf * ShiftRight8, vuf );
	return vec4( vuf * Inv255, gf * PackUpscale, bf * PackUpscale, af );
}
vec3 packDepthToRGB( const in float v ) {
	if( v <= 0.0 )
		return vec3( 0., 0., 0. );
	if( v >= 1.0 )
		return vec3( 1., 1., 1. );
	float vuf;
	float bf = modf( v * PackFactors.b, vuf );
	float gf = modf( vuf * ShiftRight8, vuf );
	return vec3( vuf * Inv255, gf * PackUpscale, bf );
}
vec2 packDepthToRG( const in float v ) {
	if( v <= 0.0 )
		return vec2( 0., 0. );
	if( v >= 1.0 )
		return vec2( 1., 1. );
	float vuf;
	float gf = modf( v * 256., vuf );
	return vec2( vuf * Inv255, gf );
}
float unpackRGBAToDepth( const in vec4 v ) {
	return dot( v, UnpackFactors4 );
}
float unpackRGBToDepth( const in vec3 v ) {
	return dot( v, UnpackFactors3 );
}
float unpackRGToDepth( const in vec2 v ) {
	return v.r * UnpackFactors2.r + v.g * UnpackFactors2.g;
}
vec4 pack2HalfToRGBA( const in vec2 v ) {
	vec4 r = vec4( v.x, fract( v.x * 255.0 ), v.y, fract( v.y * 255.0 ) );
	return vec4( r.x - r.y / 255.0, r.y, r.z - r.w / 255.0, r.w );
}
vec2 unpackRGBATo2Half( const in vec4 v ) {
	return vec2( v.x + ( v.y / 255.0 ), v.z + ( v.w / 255.0 ) );
}
float viewZToOrthographicDepth( const in float viewZ, const in float near, const in float far ) {
	return ( viewZ + near ) / ( near - far );
}
float orthographicDepthToViewZ( const in float depth, const in float near, const in float far ) {
	return depth * ( near - far ) - near;
}
float viewZToPerspectiveDepth( const in float viewZ, const in float near, const in float far ) {
	return ( ( near + viewZ ) * far ) / ( ( far - near ) * viewZ );
}
float perspectiveDepthToViewZ( const in float depth, const in float near, const in float far ) {
	return ( near * far ) / ( ( far - near ) * depth - far );
}`,mp=`#ifdef PREMULTIPLIED_ALPHA
	gl_FragColor.rgb *= gl_FragColor.a;
#endif`,gp=`vec4 mvPosition = vec4( transformed, 1.0 );
#ifdef USE_BATCHING
	mvPosition = batchingMatrix * mvPosition;
#endif
#ifdef USE_INSTANCING
	mvPosition = instanceMatrix * mvPosition;
#endif
mvPosition = modelViewMatrix * mvPosition;
gl_Position = projectionMatrix * mvPosition;`,_p=`#ifdef DITHERING
	gl_FragColor.rgb = dithering( gl_FragColor.rgb );
#endif`,xp=`#ifdef DITHERING
	vec3 dithering( vec3 color ) {
		float grid_position = rand( gl_FragCoord.xy );
		vec3 dither_shift_RGB = vec3( 0.25 / 255.0, -0.25 / 255.0, 0.25 / 255.0 );
		dither_shift_RGB = mix( 2.0 * dither_shift_RGB, -2.0 * dither_shift_RGB, grid_position );
		return color + dither_shift_RGB;
	}
#endif`,yp=`float roughnessFactor = roughness;
#ifdef USE_ROUGHNESSMAP
	vec4 texelRoughness = texture2D( roughnessMap, vRoughnessMapUv );
	roughnessFactor *= texelRoughness.g;
#endif`,vp=`#ifdef USE_ROUGHNESSMAP
	uniform sampler2D roughnessMap;
#endif`,Mp=`#if NUM_SPOT_LIGHT_COORDS > 0
	varying vec4 vSpotLightCoord[ NUM_SPOT_LIGHT_COORDS ];
#endif
#if NUM_SPOT_LIGHT_MAPS > 0
	uniform sampler2D spotLightMap[ NUM_SPOT_LIGHT_MAPS ];
#endif
#ifdef USE_SHADOWMAP
	#if NUM_DIR_LIGHT_SHADOWS > 0
		uniform sampler2D directionalShadowMap[ NUM_DIR_LIGHT_SHADOWS ];
		varying vec4 vDirectionalShadowCoord[ NUM_DIR_LIGHT_SHADOWS ];
		struct DirectionalLightShadow {
			float shadowIntensity;
			float shadowBias;
			float shadowNormalBias;
			float shadowRadius;
			vec2 shadowMapSize;
		};
		uniform DirectionalLightShadow directionalLightShadows[ NUM_DIR_LIGHT_SHADOWS ];
	#endif
	#if NUM_SPOT_LIGHT_SHADOWS > 0
		uniform sampler2D spotShadowMap[ NUM_SPOT_LIGHT_SHADOWS ];
		struct SpotLightShadow {
			float shadowIntensity;
			float shadowBias;
			float shadowNormalBias;
			float shadowRadius;
			vec2 shadowMapSize;
		};
		uniform SpotLightShadow spotLightShadows[ NUM_SPOT_LIGHT_SHADOWS ];
	#endif
	#if NUM_POINT_LIGHT_SHADOWS > 0
		uniform sampler2D pointShadowMap[ NUM_POINT_LIGHT_SHADOWS ];
		varying vec4 vPointShadowCoord[ NUM_POINT_LIGHT_SHADOWS ];
		struct PointLightShadow {
			float shadowIntensity;
			float shadowBias;
			float shadowNormalBias;
			float shadowRadius;
			vec2 shadowMapSize;
			float shadowCameraNear;
			float shadowCameraFar;
		};
		uniform PointLightShadow pointLightShadows[ NUM_POINT_LIGHT_SHADOWS ];
	#endif
	float texture2DCompare( sampler2D depths, vec2 uv, float compare ) {
		return step( compare, unpackRGBAToDepth( texture2D( depths, uv ) ) );
	}
	vec2 texture2DDistribution( sampler2D shadow, vec2 uv ) {
		return unpackRGBATo2Half( texture2D( shadow, uv ) );
	}
	float VSMShadow (sampler2D shadow, vec2 uv, float compare ){
		float occlusion = 1.0;
		vec2 distribution = texture2DDistribution( shadow, uv );
		float hard_shadow = step( compare , distribution.x );
		if (hard_shadow != 1.0 ) {
			float distance = compare - distribution.x ;
			float variance = max( 0.00000, distribution.y * distribution.y );
			float softness_probability = variance / (variance + distance * distance );			softness_probability = clamp( ( softness_probability - 0.3 ) / ( 0.95 - 0.3 ), 0.0, 1.0 );			occlusion = clamp( max( hard_shadow, softness_probability ), 0.0, 1.0 );
		}
		return occlusion;
	}
	float getShadow( sampler2D shadowMap, vec2 shadowMapSize, float shadowIntensity, float shadowBias, float shadowRadius, vec4 shadowCoord ) {
		float shadow = 1.0;
		shadowCoord.xyz /= shadowCoord.w;
		shadowCoord.z += shadowBias;
		bool inFrustum = shadowCoord.x >= 0.0 && shadowCoord.x <= 1.0 && shadowCoord.y >= 0.0 && shadowCoord.y <= 1.0;
		bool frustumTest = inFrustum && shadowCoord.z <= 1.0;
		if ( frustumTest ) {
		#if defined( SHADOWMAP_TYPE_PCF )
			vec2 texelSize = vec2( 1.0 ) / shadowMapSize;
			float dx0 = - texelSize.x * shadowRadius;
			float dy0 = - texelSize.y * shadowRadius;
			float dx1 = + texelSize.x * shadowRadius;
			float dy1 = + texelSize.y * shadowRadius;
			float dx2 = dx0 / 2.0;
			float dy2 = dy0 / 2.0;
			float dx3 = dx1 / 2.0;
			float dy3 = dy1 / 2.0;
			shadow = (
				texture2DCompare( shadowMap, shadowCoord.xy + vec2( dx0, dy0 ), shadowCoord.z ) +
				texture2DCompare( shadowMap, shadowCoord.xy + vec2( 0.0, dy0 ), shadowCoord.z ) +
				texture2DCompare( shadowMap, shadowCoord.xy + vec2( dx1, dy0 ), shadowCoord.z ) +
				texture2DCompare( shadowMap, shadowCoord.xy + vec2( dx2, dy2 ), shadowCoord.z ) +
				texture2DCompare( shadowMap, shadowCoord.xy + vec2( 0.0, dy2 ), shadowCoord.z ) +
				texture2DCompare( shadowMap, shadowCoord.xy + vec2( dx3, dy2 ), shadowCoord.z ) +
				texture2DCompare( shadowMap, shadowCoord.xy + vec2( dx0, 0.0 ), shadowCoord.z ) +
				texture2DCompare( shadowMap, shadowCoord.xy + vec2( dx2, 0.0 ), shadowCoord.z ) +
				texture2DCompare( shadowMap, shadowCoord.xy, shadowCoord.z ) +
				texture2DCompare( shadowMap, shadowCoord.xy + vec2( dx3, 0.0 ), shadowCoord.z ) +
				texture2DCompare( shadowMap, shadowCoord.xy + vec2( dx1, 0.0 ), shadowCoord.z ) +
				texture2DCompare( shadowMap, shadowCoord.xy + vec2( dx2, dy3 ), shadowCoord.z ) +
				texture2DCompare( shadowMap, shadowCoord.xy + vec2( 0.0, dy3 ), shadowCoord.z ) +
				texture2DCompare( shadowMap, shadowCoord.xy + vec2( dx3, dy3 ), shadowCoord.z ) +
				texture2DCompare( shadowMap, shadowCoord.xy + vec2( dx0, dy1 ), shadowCoord.z ) +
				texture2DCompare( shadowMap, shadowCoord.xy + vec2( 0.0, dy1 ), shadowCoord.z ) +
				texture2DCompare( shadowMap, shadowCoord.xy + vec2( dx1, dy1 ), shadowCoord.z )
			) * ( 1.0 / 17.0 );
		#elif defined( SHADOWMAP_TYPE_PCF_SOFT )
			vec2 texelSize = vec2( 1.0 ) / shadowMapSize;
			float dx = texelSize.x;
			float dy = texelSize.y;
			vec2 uv = shadowCoord.xy;
			vec2 f = fract( uv * shadowMapSize + 0.5 );
			uv -= f * texelSize;
			shadow = (
				texture2DCompare( shadowMap, uv, shadowCoord.z ) +
				texture2DCompare( shadowMap, uv + vec2( dx, 0.0 ), shadowCoord.z ) +
				texture2DCompare( shadowMap, uv + vec2( 0.0, dy ), shadowCoord.z ) +
				texture2DCompare( shadowMap, uv + texelSize, shadowCoord.z ) +
				mix( texture2DCompare( shadowMap, uv + vec2( -dx, 0.0 ), shadowCoord.z ),
					 texture2DCompare( shadowMap, uv + vec2( 2.0 * dx, 0.0 ), shadowCoord.z ),
					 f.x ) +
				mix( texture2DCompare( shadowMap, uv + vec2( -dx, dy ), shadowCoord.z ),
					 texture2DCompare( shadowMap, uv + vec2( 2.0 * dx, dy ), shadowCoord.z ),
					 f.x ) +
				mix( texture2DCompare( shadowMap, uv + vec2( 0.0, -dy ), shadowCoord.z ),
					 texture2DCompare( shadowMap, uv + vec2( 0.0, 2.0 * dy ), shadowCoord.z ),
					 f.y ) +
				mix( texture2DCompare( shadowMap, uv + vec2( dx, -dy ), shadowCoord.z ),
					 texture2DCompare( shadowMap, uv + vec2( dx, 2.0 * dy ), shadowCoord.z ),
					 f.y ) +
				mix( mix( texture2DCompare( shadowMap, uv + vec2( -dx, -dy ), shadowCoord.z ),
						  texture2DCompare( shadowMap, uv + vec2( 2.0 * dx, -dy ), shadowCoord.z ),
						  f.x ),
					 mix( texture2DCompare( shadowMap, uv + vec2( -dx, 2.0 * dy ), shadowCoord.z ),
						  texture2DCompare( shadowMap, uv + vec2( 2.0 * dx, 2.0 * dy ), shadowCoord.z ),
						  f.x ),
					 f.y )
			) * ( 1.0 / 9.0 );
		#elif defined( SHADOWMAP_TYPE_VSM )
			shadow = VSMShadow( shadowMap, shadowCoord.xy, shadowCoord.z );
		#else
			shadow = texture2DCompare( shadowMap, shadowCoord.xy, shadowCoord.z );
		#endif
		}
		return mix( 1.0, shadow, shadowIntensity );
	}
	vec2 cubeToUV( vec3 v, float texelSizeY ) {
		vec3 absV = abs( v );
		float scaleToCube = 1.0 / max( absV.x, max( absV.y, absV.z ) );
		absV *= scaleToCube;
		v *= scaleToCube * ( 1.0 - 2.0 * texelSizeY );
		vec2 planar = v.xy;
		float almostATexel = 1.5 * texelSizeY;
		float almostOne = 1.0 - almostATexel;
		if ( absV.z >= almostOne ) {
			if ( v.z > 0.0 )
				planar.x = 4.0 - v.x;
		} else if ( absV.x >= almostOne ) {
			float signX = sign( v.x );
			planar.x = v.z * signX + 2.0 * signX;
		} else if ( absV.y >= almostOne ) {
			float signY = sign( v.y );
			planar.x = v.x + 2.0 * signY + 2.0;
			planar.y = v.z * signY - 2.0;
		}
		return vec2( 0.125, 0.25 ) * planar + vec2( 0.375, 0.75 );
	}
	float getPointShadow( sampler2D shadowMap, vec2 shadowMapSize, float shadowIntensity, float shadowBias, float shadowRadius, vec4 shadowCoord, float shadowCameraNear, float shadowCameraFar ) {
		float shadow = 1.0;
		vec3 lightToPosition = shadowCoord.xyz;
		
		float lightToPositionLength = length( lightToPosition );
		if ( lightToPositionLength - shadowCameraFar <= 0.0 && lightToPositionLength - shadowCameraNear >= 0.0 ) {
			float dp = ( lightToPositionLength - shadowCameraNear ) / ( shadowCameraFar - shadowCameraNear );			dp += shadowBias;
			vec3 bd3D = normalize( lightToPosition );
			vec2 texelSize = vec2( 1.0 ) / ( shadowMapSize * vec2( 4.0, 2.0 ) );
			#if defined( SHADOWMAP_TYPE_PCF ) || defined( SHADOWMAP_TYPE_PCF_SOFT ) || defined( SHADOWMAP_TYPE_VSM )
				vec2 offset = vec2( - 1, 1 ) * shadowRadius * texelSize.y;
				shadow = (
					texture2DCompare( shadowMap, cubeToUV( bd3D + offset.xyy, texelSize.y ), dp ) +
					texture2DCompare( shadowMap, cubeToUV( bd3D + offset.yyy, texelSize.y ), dp ) +
					texture2DCompare( shadowMap, cubeToUV( bd3D + offset.xyx, texelSize.y ), dp ) +
					texture2DCompare( shadowMap, cubeToUV( bd3D + offset.yyx, texelSize.y ), dp ) +
					texture2DCompare( shadowMap, cubeToUV( bd3D, texelSize.y ), dp ) +
					texture2DCompare( shadowMap, cubeToUV( bd3D + offset.xxy, texelSize.y ), dp ) +
					texture2DCompare( shadowMap, cubeToUV( bd3D + offset.yxy, texelSize.y ), dp ) +
					texture2DCompare( shadowMap, cubeToUV( bd3D + offset.xxx, texelSize.y ), dp ) +
					texture2DCompare( shadowMap, cubeToUV( bd3D + offset.yxx, texelSize.y ), dp )
				) * ( 1.0 / 9.0 );
			#else
				shadow = texture2DCompare( shadowMap, cubeToUV( bd3D, texelSize.y ), dp );
			#endif
		}
		return mix( 1.0, shadow, shadowIntensity );
	}
#endif`,Sp=`#if NUM_SPOT_LIGHT_COORDS > 0
	uniform mat4 spotLightMatrix[ NUM_SPOT_LIGHT_COORDS ];
	varying vec4 vSpotLightCoord[ NUM_SPOT_LIGHT_COORDS ];
#endif
#ifdef USE_SHADOWMAP
	#if NUM_DIR_LIGHT_SHADOWS > 0
		uniform mat4 directionalShadowMatrix[ NUM_DIR_LIGHT_SHADOWS ];
		varying vec4 vDirectionalShadowCoord[ NUM_DIR_LIGHT_SHADOWS ];
		struct DirectionalLightShadow {
			float shadowIntensity;
			float shadowBias;
			float shadowNormalBias;
			float shadowRadius;
			vec2 shadowMapSize;
		};
		uniform DirectionalLightShadow directionalLightShadows[ NUM_DIR_LIGHT_SHADOWS ];
	#endif
	#if NUM_SPOT_LIGHT_SHADOWS > 0
		struct SpotLightShadow {
			float shadowIntensity;
			float shadowBias;
			float shadowNormalBias;
			float shadowRadius;
			vec2 shadowMapSize;
		};
		uniform SpotLightShadow spotLightShadows[ NUM_SPOT_LIGHT_SHADOWS ];
	#endif
	#if NUM_POINT_LIGHT_SHADOWS > 0
		uniform mat4 pointShadowMatrix[ NUM_POINT_LIGHT_SHADOWS ];
		varying vec4 vPointShadowCoord[ NUM_POINT_LIGHT_SHADOWS ];
		struct PointLightShadow {
			float shadowIntensity;
			float shadowBias;
			float shadowNormalBias;
			float shadowRadius;
			vec2 shadowMapSize;
			float shadowCameraNear;
			float shadowCameraFar;
		};
		uniform PointLightShadow pointLightShadows[ NUM_POINT_LIGHT_SHADOWS ];
	#endif
#endif`,bp=`#if ( defined( USE_SHADOWMAP ) && ( NUM_DIR_LIGHT_SHADOWS > 0 || NUM_POINT_LIGHT_SHADOWS > 0 ) ) || ( NUM_SPOT_LIGHT_COORDS > 0 )
	vec3 shadowWorldNormal = inverseTransformDirection( transformedNormal, viewMatrix );
	vec4 shadowWorldPosition;
#endif
#if defined( USE_SHADOWMAP )
	#if NUM_DIR_LIGHT_SHADOWS > 0
		#pragma unroll_loop_start
		for ( int i = 0; i < NUM_DIR_LIGHT_SHADOWS; i ++ ) {
			shadowWorldPosition = worldPosition + vec4( shadowWorldNormal * directionalLightShadows[ i ].shadowNormalBias, 0 );
			vDirectionalShadowCoord[ i ] = directionalShadowMatrix[ i ] * shadowWorldPosition;
		}
		#pragma unroll_loop_end
	#endif
	#if NUM_POINT_LIGHT_SHADOWS > 0
		#pragma unroll_loop_start
		for ( int i = 0; i < NUM_POINT_LIGHT_SHADOWS; i ++ ) {
			shadowWorldPosition = worldPosition + vec4( shadowWorldNormal * pointLightShadows[ i ].shadowNormalBias, 0 );
			vPointShadowCoord[ i ] = pointShadowMatrix[ i ] * shadowWorldPosition;
		}
		#pragma unroll_loop_end
	#endif
#endif
#if NUM_SPOT_LIGHT_COORDS > 0
	#pragma unroll_loop_start
	for ( int i = 0; i < NUM_SPOT_LIGHT_COORDS; i ++ ) {
		shadowWorldPosition = worldPosition;
		#if ( defined( USE_SHADOWMAP ) && UNROLLED_LOOP_INDEX < NUM_SPOT_LIGHT_SHADOWS )
			shadowWorldPosition.xyz += shadowWorldNormal * spotLightShadows[ i ].shadowNormalBias;
		#endif
		vSpotLightCoord[ i ] = spotLightMatrix[ i ] * shadowWorldPosition;
	}
	#pragma unroll_loop_end
#endif`,Ep=`float getShadowMask() {
	float shadow = 1.0;
	#ifdef USE_SHADOWMAP
	#if NUM_DIR_LIGHT_SHADOWS > 0
	DirectionalLightShadow directionalLight;
	#pragma unroll_loop_start
	for ( int i = 0; i < NUM_DIR_LIGHT_SHADOWS; i ++ ) {
		directionalLight = directionalLightShadows[ i ];
		shadow *= receiveShadow ? getShadow( directionalShadowMap[ i ], directionalLight.shadowMapSize, directionalLight.shadowIntensity, directionalLight.shadowBias, directionalLight.shadowRadius, vDirectionalShadowCoord[ i ] ) : 1.0;
	}
	#pragma unroll_loop_end
	#endif
	#if NUM_SPOT_LIGHT_SHADOWS > 0
	SpotLightShadow spotLight;
	#pragma unroll_loop_start
	for ( int i = 0; i < NUM_SPOT_LIGHT_SHADOWS; i ++ ) {
		spotLight = spotLightShadows[ i ];
		shadow *= receiveShadow ? getShadow( spotShadowMap[ i ], spotLight.shadowMapSize, spotLight.shadowIntensity, spotLight.shadowBias, spotLight.shadowRadius, vSpotLightCoord[ i ] ) : 1.0;
	}
	#pragma unroll_loop_end
	#endif
	#if NUM_POINT_LIGHT_SHADOWS > 0
	PointLightShadow pointLight;
	#pragma unroll_loop_start
	for ( int i = 0; i < NUM_POINT_LIGHT_SHADOWS; i ++ ) {
		pointLight = pointLightShadows[ i ];
		shadow *= receiveShadow ? getPointShadow( pointShadowMap[ i ], pointLight.shadowMapSize, pointLight.shadowIntensity, pointLight.shadowBias, pointLight.shadowRadius, vPointShadowCoord[ i ], pointLight.shadowCameraNear, pointLight.shadowCameraFar ) : 1.0;
	}
	#pragma unroll_loop_end
	#endif
	#endif
	return shadow;
}`,Tp=`#ifdef USE_SKINNING
	mat4 boneMatX = getBoneMatrix( skinIndex.x );
	mat4 boneMatY = getBoneMatrix( skinIndex.y );
	mat4 boneMatZ = getBoneMatrix( skinIndex.z );
	mat4 boneMatW = getBoneMatrix( skinIndex.w );
#endif`,wp=`#ifdef USE_SKINNING
	uniform mat4 bindMatrix;
	uniform mat4 bindMatrixInverse;
	uniform highp sampler2D boneTexture;
	mat4 getBoneMatrix( const in float i ) {
		int size = textureSize( boneTexture, 0 ).x;
		int j = int( i ) * 4;
		int x = j % size;
		int y = j / size;
		vec4 v1 = texelFetch( boneTexture, ivec2( x, y ), 0 );
		vec4 v2 = texelFetch( boneTexture, ivec2( x + 1, y ), 0 );
		vec4 v3 = texelFetch( boneTexture, ivec2( x + 2, y ), 0 );
		vec4 v4 = texelFetch( boneTexture, ivec2( x + 3, y ), 0 );
		return mat4( v1, v2, v3, v4 );
	}
#endif`,Ap=`#ifdef USE_SKINNING
	vec4 skinVertex = bindMatrix * vec4( transformed, 1.0 );
	vec4 skinned = vec4( 0.0 );
	skinned += boneMatX * skinVertex * skinWeight.x;
	skinned += boneMatY * skinVertex * skinWeight.y;
	skinned += boneMatZ * skinVertex * skinWeight.z;
	skinned += boneMatW * skinVertex * skinWeight.w;
	transformed = ( bindMatrixInverse * skinned ).xyz;
#endif`,Cp=`#ifdef USE_SKINNING
	mat4 skinMatrix = mat4( 0.0 );
	skinMatrix += skinWeight.x * boneMatX;
	skinMatrix += skinWeight.y * boneMatY;
	skinMatrix += skinWeight.z * boneMatZ;
	skinMatrix += skinWeight.w * boneMatW;
	skinMatrix = bindMatrixInverse * skinMatrix * bindMatrix;
	objectNormal = vec4( skinMatrix * vec4( objectNormal, 0.0 ) ).xyz;
	#ifdef USE_TANGENT
		objectTangent = vec4( skinMatrix * vec4( objectTangent, 0.0 ) ).xyz;
	#endif
#endif`,Rp=`float specularStrength;
#ifdef USE_SPECULARMAP
	vec4 texelSpecular = texture2D( specularMap, vSpecularMapUv );
	specularStrength = texelSpecular.r;
#else
	specularStrength = 1.0;
#endif`,Pp=`#ifdef USE_SPECULARMAP
	uniform sampler2D specularMap;
#endif`,Ip=`#if defined( TONE_MAPPING )
	gl_FragColor.rgb = toneMapping( gl_FragColor.rgb );
#endif`,Lp=`#ifndef saturate
#define saturate( a ) clamp( a, 0.0, 1.0 )
#endif
uniform float toneMappingExposure;
vec3 LinearToneMapping( vec3 color ) {
	return saturate( toneMappingExposure * color );
}
vec3 ReinhardToneMapping( vec3 color ) {
	color *= toneMappingExposure;
	return saturate( color / ( vec3( 1.0 ) + color ) );
}
vec3 CineonToneMapping( vec3 color ) {
	color *= toneMappingExposure;
	color = max( vec3( 0.0 ), color - 0.004 );
	return pow( ( color * ( 6.2 * color + 0.5 ) ) / ( color * ( 6.2 * color + 1.7 ) + 0.06 ), vec3( 2.2 ) );
}
vec3 RRTAndODTFit( vec3 v ) {
	vec3 a = v * ( v + 0.0245786 ) - 0.000090537;
	vec3 b = v * ( 0.983729 * v + 0.4329510 ) + 0.238081;
	return a / b;
}
vec3 ACESFilmicToneMapping( vec3 color ) {
	const mat3 ACESInputMat = mat3(
		vec3( 0.59719, 0.07600, 0.02840 ),		vec3( 0.35458, 0.90834, 0.13383 ),
		vec3( 0.04823, 0.01566, 0.83777 )
	);
	const mat3 ACESOutputMat = mat3(
		vec3(  1.60475, -0.10208, -0.00327 ),		vec3( -0.53108,  1.10813, -0.07276 ),
		vec3( -0.07367, -0.00605,  1.07602 )
	);
	color *= toneMappingExposure / 0.6;
	color = ACESInputMat * color;
	color = RRTAndODTFit( color );
	color = ACESOutputMat * color;
	return saturate( color );
}
const mat3 LINEAR_REC2020_TO_LINEAR_SRGB = mat3(
	vec3( 1.6605, - 0.1246, - 0.0182 ),
	vec3( - 0.5876, 1.1329, - 0.1006 ),
	vec3( - 0.0728, - 0.0083, 1.1187 )
);
const mat3 LINEAR_SRGB_TO_LINEAR_REC2020 = mat3(
	vec3( 0.6274, 0.0691, 0.0164 ),
	vec3( 0.3293, 0.9195, 0.0880 ),
	vec3( 0.0433, 0.0113, 0.8956 )
);
vec3 agxDefaultContrastApprox( vec3 x ) {
	vec3 x2 = x * x;
	vec3 x4 = x2 * x2;
	return + 15.5 * x4 * x2
		- 40.14 * x4 * x
		+ 31.96 * x4
		- 6.868 * x2 * x
		+ 0.4298 * x2
		+ 0.1191 * x
		- 0.00232;
}
vec3 AgXToneMapping( vec3 color ) {
	const mat3 AgXInsetMatrix = mat3(
		vec3( 0.856627153315983, 0.137318972929847, 0.11189821299995 ),
		vec3( 0.0951212405381588, 0.761241990602591, 0.0767994186031903 ),
		vec3( 0.0482516061458583, 0.101439036467562, 0.811302368396859 )
	);
	const mat3 AgXOutsetMatrix = mat3(
		vec3( 1.1271005818144368, - 0.1413297634984383, - 0.14132976349843826 ),
		vec3( - 0.11060664309660323, 1.157823702216272, - 0.11060664309660294 ),
		vec3( - 0.016493938717834573, - 0.016493938717834257, 1.2519364065950405 )
	);
	const float AgxMinEv = - 12.47393;	const float AgxMaxEv = 4.026069;
	color *= toneMappingExposure;
	color = LINEAR_SRGB_TO_LINEAR_REC2020 * color;
	color = AgXInsetMatrix * color;
	color = max( color, 1e-10 );	color = log2( color );
	color = ( color - AgxMinEv ) / ( AgxMaxEv - AgxMinEv );
	color = clamp( color, 0.0, 1.0 );
	color = agxDefaultContrastApprox( color );
	color = AgXOutsetMatrix * color;
	color = pow( max( vec3( 0.0 ), color ), vec3( 2.2 ) );
	color = LINEAR_REC2020_TO_LINEAR_SRGB * color;
	color = clamp( color, 0.0, 1.0 );
	return color;
}
vec3 NeutralToneMapping( vec3 color ) {
	const float StartCompression = 0.8 - 0.04;
	const float Desaturation = 0.15;
	color *= toneMappingExposure;
	float x = min( color.r, min( color.g, color.b ) );
	float offset = x < 0.08 ? x - 6.25 * x * x : 0.04;
	color -= offset;
	float peak = max( color.r, max( color.g, color.b ) );
	if ( peak < StartCompression ) return color;
	float d = 1. - StartCompression;
	float newPeak = 1. - d * d / ( peak + d - StartCompression );
	color *= newPeak / peak;
	float g = 1. - 1. / ( Desaturation * ( peak - newPeak ) + 1. );
	return mix( color, vec3( newPeak ), g );
}
vec3 CustomToneMapping( vec3 color ) { return color; }`,Dp=`#ifdef USE_TRANSMISSION
	material.transmission = transmission;
	material.transmissionAlpha = 1.0;
	material.thickness = thickness;
	material.attenuationDistance = attenuationDistance;
	material.attenuationColor = attenuationColor;
	#ifdef USE_TRANSMISSIONMAP
		material.transmission *= texture2D( transmissionMap, vTransmissionMapUv ).r;
	#endif
	#ifdef USE_THICKNESSMAP
		material.thickness *= texture2D( thicknessMap, vThicknessMapUv ).g;
	#endif
	vec3 pos = vWorldPosition;
	vec3 v = normalize( cameraPosition - pos );
	vec3 n = inverseTransformDirection( normal, viewMatrix );
	vec4 transmitted = getIBLVolumeRefraction(
		n, v, material.roughness, material.diffuseColor, material.specularColor, material.specularF90,
		pos, modelMatrix, viewMatrix, projectionMatrix, material.dispersion, material.ior, material.thickness,
		material.attenuationColor, material.attenuationDistance );
	material.transmissionAlpha = mix( material.transmissionAlpha, transmitted.a, material.transmission );
	totalDiffuse = mix( totalDiffuse, transmitted.rgb, material.transmission );
#endif`,Up=`#ifdef USE_TRANSMISSION
	uniform float transmission;
	uniform float thickness;
	uniform float attenuationDistance;
	uniform vec3 attenuationColor;
	#ifdef USE_TRANSMISSIONMAP
		uniform sampler2D transmissionMap;
	#endif
	#ifdef USE_THICKNESSMAP
		uniform sampler2D thicknessMap;
	#endif
	uniform vec2 transmissionSamplerSize;
	uniform sampler2D transmissionSamplerMap;
	uniform mat4 modelMatrix;
	uniform mat4 projectionMatrix;
	varying vec3 vWorldPosition;
	float w0( float a ) {
		return ( 1.0 / 6.0 ) * ( a * ( a * ( - a + 3.0 ) - 3.0 ) + 1.0 );
	}
	float w1( float a ) {
		return ( 1.0 / 6.0 ) * ( a *  a * ( 3.0 * a - 6.0 ) + 4.0 );
	}
	float w2( float a ){
		return ( 1.0 / 6.0 ) * ( a * ( a * ( - 3.0 * a + 3.0 ) + 3.0 ) + 1.0 );
	}
	float w3( float a ) {
		return ( 1.0 / 6.0 ) * ( a * a * a );
	}
	float g0( float a ) {
		return w0( a ) + w1( a );
	}
	float g1( float a ) {
		return w2( a ) + w3( a );
	}
	float h0( float a ) {
		return - 1.0 + w1( a ) / ( w0( a ) + w1( a ) );
	}
	float h1( float a ) {
		return 1.0 + w3( a ) / ( w2( a ) + w3( a ) );
	}
	vec4 bicubic( sampler2D tex, vec2 uv, vec4 texelSize, float lod ) {
		uv = uv * texelSize.zw + 0.5;
		vec2 iuv = floor( uv );
		vec2 fuv = fract( uv );
		float g0x = g0( fuv.x );
		float g1x = g1( fuv.x );
		float h0x = h0( fuv.x );
		float h1x = h1( fuv.x );
		float h0y = h0( fuv.y );
		float h1y = h1( fuv.y );
		vec2 p0 = ( vec2( iuv.x + h0x, iuv.y + h0y ) - 0.5 ) * texelSize.xy;
		vec2 p1 = ( vec2( iuv.x + h1x, iuv.y + h0y ) - 0.5 ) * texelSize.xy;
		vec2 p2 = ( vec2( iuv.x + h0x, iuv.y + h1y ) - 0.5 ) * texelSize.xy;
		vec2 p3 = ( vec2( iuv.x + h1x, iuv.y + h1y ) - 0.5 ) * texelSize.xy;
		return g0( fuv.y ) * ( g0x * textureLod( tex, p0, lod ) + g1x * textureLod( tex, p1, lod ) ) +
			g1( fuv.y ) * ( g0x * textureLod( tex, p2, lod ) + g1x * textureLod( tex, p3, lod ) );
	}
	vec4 textureBicubic( sampler2D sampler, vec2 uv, float lod ) {
		vec2 fLodSize = vec2( textureSize( sampler, int( lod ) ) );
		vec2 cLodSize = vec2( textureSize( sampler, int( lod + 1.0 ) ) );
		vec2 fLodSizeInv = 1.0 / fLodSize;
		vec2 cLodSizeInv = 1.0 / cLodSize;
		vec4 fSample = bicubic( sampler, uv, vec4( fLodSizeInv, fLodSize ), floor( lod ) );
		vec4 cSample = bicubic( sampler, uv, vec4( cLodSizeInv, cLodSize ), ceil( lod ) );
		return mix( fSample, cSample, fract( lod ) );
	}
	vec3 getVolumeTransmissionRay( const in vec3 n, const in vec3 v, const in float thickness, const in float ior, const in mat4 modelMatrix ) {
		vec3 refractionVector = refract( - v, normalize( n ), 1.0 / ior );
		vec3 modelScale;
		modelScale.x = length( vec3( modelMatrix[ 0 ].xyz ) );
		modelScale.y = length( vec3( modelMatrix[ 1 ].xyz ) );
		modelScale.z = length( vec3( modelMatrix[ 2 ].xyz ) );
		return normalize( refractionVector ) * thickness * modelScale;
	}
	float applyIorToRoughness( const in float roughness, const in float ior ) {
		return roughness * clamp( ior * 2.0 - 2.0, 0.0, 1.0 );
	}
	vec4 getTransmissionSample( const in vec2 fragCoord, const in float roughness, const in float ior ) {
		float lod = log2( transmissionSamplerSize.x ) * applyIorToRoughness( roughness, ior );
		return textureBicubic( transmissionSamplerMap, fragCoord.xy, lod );
	}
	vec3 volumeAttenuation( const in float transmissionDistance, const in vec3 attenuationColor, const in float attenuationDistance ) {
		if ( isinf( attenuationDistance ) ) {
			return vec3( 1.0 );
		} else {
			vec3 attenuationCoefficient = -log( attenuationColor ) / attenuationDistance;
			vec3 transmittance = exp( - attenuationCoefficient * transmissionDistance );			return transmittance;
		}
	}
	vec4 getIBLVolumeRefraction( const in vec3 n, const in vec3 v, const in float roughness, const in vec3 diffuseColor,
		const in vec3 specularColor, const in float specularF90, const in vec3 position, const in mat4 modelMatrix,
		const in mat4 viewMatrix, const in mat4 projMatrix, const in float dispersion, const in float ior, const in float thickness,
		const in vec3 attenuationColor, const in float attenuationDistance ) {
		vec4 transmittedLight;
		vec3 transmittance;
		#ifdef USE_DISPERSION
			float halfSpread = ( ior - 1.0 ) * 0.025 * dispersion;
			vec3 iors = vec3( ior - halfSpread, ior, ior + halfSpread );
			for ( int i = 0; i < 3; i ++ ) {
				vec3 transmissionRay = getVolumeTransmissionRay( n, v, thickness, iors[ i ], modelMatrix );
				vec3 refractedRayExit = position + transmissionRay;
		
				vec4 ndcPos = projMatrix * viewMatrix * vec4( refractedRayExit, 1.0 );
				vec2 refractionCoords = ndcPos.xy / ndcPos.w;
				refractionCoords += 1.0;
				refractionCoords /= 2.0;
		
				vec4 transmissionSample = getTransmissionSample( refractionCoords, roughness, iors[ i ] );
				transmittedLight[ i ] = transmissionSample[ i ];
				transmittedLight.a += transmissionSample.a;
				transmittance[ i ] = diffuseColor[ i ] * volumeAttenuation( length( transmissionRay ), attenuationColor, attenuationDistance )[ i ];
			}
			transmittedLight.a /= 3.0;
		
		#else
		
			vec3 transmissionRay = getVolumeTransmissionRay( n, v, thickness, ior, modelMatrix );
			vec3 refractedRayExit = position + transmissionRay;
			vec4 ndcPos = projMatrix * viewMatrix * vec4( refractedRayExit, 1.0 );
			vec2 refractionCoords = ndcPos.xy / ndcPos.w;
			refractionCoords += 1.0;
			refractionCoords /= 2.0;
			transmittedLight = getTransmissionSample( refractionCoords, roughness, ior );
			transmittance = diffuseColor * volumeAttenuation( length( transmissionRay ), attenuationColor, attenuationDistance );
		
		#endif
		vec3 attenuatedColor = transmittance * transmittedLight.rgb;
		vec3 F = EnvironmentBRDF( n, v, specularColor, specularF90, roughness );
		float transmittanceFactor = ( transmittance.r + transmittance.g + transmittance.b ) / 3.0;
		return vec4( ( 1.0 - F ) * attenuatedColor, 1.0 - ( 1.0 - transmittedLight.a ) * transmittanceFactor );
	}
#endif`,Np=`#if defined( USE_UV ) || defined( USE_ANISOTROPY )
	varying vec2 vUv;
#endif
#ifdef USE_MAP
	varying vec2 vMapUv;
#endif
#ifdef USE_ALPHAMAP
	varying vec2 vAlphaMapUv;
#endif
#ifdef USE_LIGHTMAP
	varying vec2 vLightMapUv;
#endif
#ifdef USE_AOMAP
	varying vec2 vAoMapUv;
#endif
#ifdef USE_BUMPMAP
	varying vec2 vBumpMapUv;
#endif
#ifdef USE_NORMALMAP
	varying vec2 vNormalMapUv;
#endif
#ifdef USE_EMISSIVEMAP
	varying vec2 vEmissiveMapUv;
#endif
#ifdef USE_METALNESSMAP
	varying vec2 vMetalnessMapUv;
#endif
#ifdef USE_ROUGHNESSMAP
	varying vec2 vRoughnessMapUv;
#endif
#ifdef USE_ANISOTROPYMAP
	varying vec2 vAnisotropyMapUv;
#endif
#ifdef USE_CLEARCOATMAP
	varying vec2 vClearcoatMapUv;
#endif
#ifdef USE_CLEARCOAT_NORMALMAP
	varying vec2 vClearcoatNormalMapUv;
#endif
#ifdef USE_CLEARCOAT_ROUGHNESSMAP
	varying vec2 vClearcoatRoughnessMapUv;
#endif
#ifdef USE_IRIDESCENCEMAP
	varying vec2 vIridescenceMapUv;
#endif
#ifdef USE_IRIDESCENCE_THICKNESSMAP
	varying vec2 vIridescenceThicknessMapUv;
#endif
#ifdef USE_SHEEN_COLORMAP
	varying vec2 vSheenColorMapUv;
#endif
#ifdef USE_SHEEN_ROUGHNESSMAP
	varying vec2 vSheenRoughnessMapUv;
#endif
#ifdef USE_SPECULARMAP
	varying vec2 vSpecularMapUv;
#endif
#ifdef USE_SPECULAR_COLORMAP
	varying vec2 vSpecularColorMapUv;
#endif
#ifdef USE_SPECULAR_INTENSITYMAP
	varying vec2 vSpecularIntensityMapUv;
#endif
#ifdef USE_TRANSMISSIONMAP
	uniform mat3 transmissionMapTransform;
	varying vec2 vTransmissionMapUv;
#endif
#ifdef USE_THICKNESSMAP
	uniform mat3 thicknessMapTransform;
	varying vec2 vThicknessMapUv;
#endif`,Fp=`#if defined( USE_UV ) || defined( USE_ANISOTROPY )
	varying vec2 vUv;
#endif
#ifdef USE_MAP
	uniform mat3 mapTransform;
	varying vec2 vMapUv;
#endif
#ifdef USE_ALPHAMAP
	uniform mat3 alphaMapTransform;
	varying vec2 vAlphaMapUv;
#endif
#ifdef USE_LIGHTMAP
	uniform mat3 lightMapTransform;
	varying vec2 vLightMapUv;
#endif
#ifdef USE_AOMAP
	uniform mat3 aoMapTransform;
	varying vec2 vAoMapUv;
#endif
#ifdef USE_BUMPMAP
	uniform mat3 bumpMapTransform;
	varying vec2 vBumpMapUv;
#endif
#ifdef USE_NORMALMAP
	uniform mat3 normalMapTransform;
	varying vec2 vNormalMapUv;
#endif
#ifdef USE_DISPLACEMENTMAP
	uniform mat3 displacementMapTransform;
	varying vec2 vDisplacementMapUv;
#endif
#ifdef USE_EMISSIVEMAP
	uniform mat3 emissiveMapTransform;
	varying vec2 vEmissiveMapUv;
#endif
#ifdef USE_METALNESSMAP
	uniform mat3 metalnessMapTransform;
	varying vec2 vMetalnessMapUv;
#endif
#ifdef USE_ROUGHNESSMAP
	uniform mat3 roughnessMapTransform;
	varying vec2 vRoughnessMapUv;
#endif
#ifdef USE_ANISOTROPYMAP
	uniform mat3 anisotropyMapTransform;
	varying vec2 vAnisotropyMapUv;
#endif
#ifdef USE_CLEARCOATMAP
	uniform mat3 clearcoatMapTransform;
	varying vec2 vClearcoatMapUv;
#endif
#ifdef USE_CLEARCOAT_NORMALMAP
	uniform mat3 clearcoatNormalMapTransform;
	varying vec2 vClearcoatNormalMapUv;
#endif
#ifdef USE_CLEARCOAT_ROUGHNESSMAP
	uniform mat3 clearcoatRoughnessMapTransform;
	varying vec2 vClearcoatRoughnessMapUv;
#endif
#ifdef USE_SHEEN_COLORMAP
	uniform mat3 sheenColorMapTransform;
	varying vec2 vSheenColorMapUv;
#endif
#ifdef USE_SHEEN_ROUGHNESSMAP
	uniform mat3 sheenRoughnessMapTransform;
	varying vec2 vSheenRoughnessMapUv;
#endif
#ifdef USE_IRIDESCENCEMAP
	uniform mat3 iridescenceMapTransform;
	varying vec2 vIridescenceMapUv;
#endif
#ifdef USE_IRIDESCENCE_THICKNESSMAP
	uniform mat3 iridescenceThicknessMapTransform;
	varying vec2 vIridescenceThicknessMapUv;
#endif
#ifdef USE_SPECULARMAP
	uniform mat3 specularMapTransform;
	varying vec2 vSpecularMapUv;
#endif
#ifdef USE_SPECULAR_COLORMAP
	uniform mat3 specularColorMapTransform;
	varying vec2 vSpecularColorMapUv;
#endif
#ifdef USE_SPECULAR_INTENSITYMAP
	uniform mat3 specularIntensityMapTransform;
	varying vec2 vSpecularIntensityMapUv;
#endif
#ifdef USE_TRANSMISSIONMAP
	uniform mat3 transmissionMapTransform;
	varying vec2 vTransmissionMapUv;
#endif
#ifdef USE_THICKNESSMAP
	uniform mat3 thicknessMapTransform;
	varying vec2 vThicknessMapUv;
#endif`,Op=`#if defined( USE_UV ) || defined( USE_ANISOTROPY )
	vUv = vec3( uv, 1 ).xy;
#endif
#ifdef USE_MAP
	vMapUv = ( mapTransform * vec3( MAP_UV, 1 ) ).xy;
#endif
#ifdef USE_ALPHAMAP
	vAlphaMapUv = ( alphaMapTransform * vec3( ALPHAMAP_UV, 1 ) ).xy;
#endif
#ifdef USE_LIGHTMAP
	vLightMapUv = ( lightMapTransform * vec3( LIGHTMAP_UV, 1 ) ).xy;
#endif
#ifdef USE_AOMAP
	vAoMapUv = ( aoMapTransform * vec3( AOMAP_UV, 1 ) ).xy;
#endif
#ifdef USE_BUMPMAP
	vBumpMapUv = ( bumpMapTransform * vec3( BUMPMAP_UV, 1 ) ).xy;
#endif
#ifdef USE_NORMALMAP
	vNormalMapUv = ( normalMapTransform * vec3( NORMALMAP_UV, 1 ) ).xy;
#endif
#ifdef USE_DISPLACEMENTMAP
	vDisplacementMapUv = ( displacementMapTransform * vec3( DISPLACEMENTMAP_UV, 1 ) ).xy;
#endif
#ifdef USE_EMISSIVEMAP
	vEmissiveMapUv = ( emissiveMapTransform * vec3( EMISSIVEMAP_UV, 1 ) ).xy;
#endif
#ifdef USE_METALNESSMAP
	vMetalnessMapUv = ( metalnessMapTransform * vec3( METALNESSMAP_UV, 1 ) ).xy;
#endif
#ifdef USE_ROUGHNESSMAP
	vRoughnessMapUv = ( roughnessMapTransform * vec3( ROUGHNESSMAP_UV, 1 ) ).xy;
#endif
#ifdef USE_ANISOTROPYMAP
	vAnisotropyMapUv = ( anisotropyMapTransform * vec3( ANISOTROPYMAP_UV, 1 ) ).xy;
#endif
#ifdef USE_CLEARCOATMAP
	vClearcoatMapUv = ( clearcoatMapTransform * vec3( CLEARCOATMAP_UV, 1 ) ).xy;
#endif
#ifdef USE_CLEARCOAT_NORMALMAP
	vClearcoatNormalMapUv = ( clearcoatNormalMapTransform * vec3( CLEARCOAT_NORMALMAP_UV, 1 ) ).xy;
#endif
#ifdef USE_CLEARCOAT_ROUGHNESSMAP
	vClearcoatRoughnessMapUv = ( clearcoatRoughnessMapTransform * vec3( CLEARCOAT_ROUGHNESSMAP_UV, 1 ) ).xy;
#endif
#ifdef USE_IRIDESCENCEMAP
	vIridescenceMapUv = ( iridescenceMapTransform * vec3( IRIDESCENCEMAP_UV, 1 ) ).xy;
#endif
#ifdef USE_IRIDESCENCE_THICKNESSMAP
	vIridescenceThicknessMapUv = ( iridescenceThicknessMapTransform * vec3( IRIDESCENCE_THICKNESSMAP_UV, 1 ) ).xy;
#endif
#ifdef USE_SHEEN_COLORMAP
	vSheenColorMapUv = ( sheenColorMapTransform * vec3( SHEEN_COLORMAP_UV, 1 ) ).xy;
#endif
#ifdef USE_SHEEN_ROUGHNESSMAP
	vSheenRoughnessMapUv = ( sheenRoughnessMapTransform * vec3( SHEEN_ROUGHNESSMAP_UV, 1 ) ).xy;
#endif
#ifdef USE_SPECULARMAP
	vSpecularMapUv = ( specularMapTransform * vec3( SPECULARMAP_UV, 1 ) ).xy;
#endif
#ifdef USE_SPECULAR_COLORMAP
	vSpecularColorMapUv = ( specularColorMapTransform * vec3( SPECULAR_COLORMAP_UV, 1 ) ).xy;
#endif
#ifdef USE_SPECULAR_INTENSITYMAP
	vSpecularIntensityMapUv = ( specularIntensityMapTransform * vec3( SPECULAR_INTENSITYMAP_UV, 1 ) ).xy;
#endif
#ifdef USE_TRANSMISSIONMAP
	vTransmissionMapUv = ( transmissionMapTransform * vec3( TRANSMISSIONMAP_UV, 1 ) ).xy;
#endif
#ifdef USE_THICKNESSMAP
	vThicknessMapUv = ( thicknessMapTransform * vec3( THICKNESSMAP_UV, 1 ) ).xy;
#endif`,Bp=`#if defined( USE_ENVMAP ) || defined( DISTANCE ) || defined ( USE_SHADOWMAP ) || defined ( USE_TRANSMISSION ) || NUM_SPOT_LIGHT_COORDS > 0
	vec4 worldPosition = vec4( transformed, 1.0 );
	#ifdef USE_BATCHING
		worldPosition = batchingMatrix * worldPosition;
	#endif
	#ifdef USE_INSTANCING
		worldPosition = instanceMatrix * worldPosition;
	#endif
	worldPosition = modelMatrix * worldPosition;
#endif`,zp=`varying vec2 vUv;
uniform mat3 uvTransform;
void main() {
	vUv = ( uvTransform * vec3( uv, 1 ) ).xy;
	gl_Position = vec4( position.xy, 1.0, 1.0 );
}`,kp=`uniform sampler2D t2D;
uniform float backgroundIntensity;
varying vec2 vUv;
void main() {
	vec4 texColor = texture2D( t2D, vUv );
	#ifdef DECODE_VIDEO_TEXTURE
		texColor = vec4( mix( pow( texColor.rgb * 0.9478672986 + vec3( 0.0521327014 ), vec3( 2.4 ) ), texColor.rgb * 0.0773993808, vec3( lessThanEqual( texColor.rgb, vec3( 0.04045 ) ) ) ), texColor.w );
	#endif
	texColor.rgb *= backgroundIntensity;
	gl_FragColor = texColor;
	#include <tonemapping_fragment>
	#include <colorspace_fragment>
}`,Hp=`varying vec3 vWorldDirection;
#include <common>
void main() {
	vWorldDirection = transformDirection( position, modelMatrix );
	#include <begin_vertex>
	#include <project_vertex>
	gl_Position.z = gl_Position.w;
}`,Vp=`#ifdef ENVMAP_TYPE_CUBE
	uniform samplerCube envMap;
#elif defined( ENVMAP_TYPE_CUBE_UV )
	uniform sampler2D envMap;
#endif
uniform float flipEnvMap;
uniform float backgroundBlurriness;
uniform float backgroundIntensity;
uniform mat3 backgroundRotation;
varying vec3 vWorldDirection;
#include <cube_uv_reflection_fragment>
void main() {
	#ifdef ENVMAP_TYPE_CUBE
		vec4 texColor = textureCube( envMap, backgroundRotation * vec3( flipEnvMap * vWorldDirection.x, vWorldDirection.yz ) );
	#elif defined( ENVMAP_TYPE_CUBE_UV )
		vec4 texColor = textureCubeUV( envMap, backgroundRotation * vWorldDirection, backgroundBlurriness );
	#else
		vec4 texColor = vec4( 0.0, 0.0, 0.0, 1.0 );
	#endif
	texColor.rgb *= backgroundIntensity;
	gl_FragColor = texColor;
	#include <tonemapping_fragment>
	#include <colorspace_fragment>
}`,Gp=`varying vec3 vWorldDirection;
#include <common>
void main() {
	vWorldDirection = transformDirection( position, modelMatrix );
	#include <begin_vertex>
	#include <project_vertex>
	gl_Position.z = gl_Position.w;
}`,Wp=`uniform samplerCube tCube;
uniform float tFlip;
uniform float opacity;
varying vec3 vWorldDirection;
void main() {
	vec4 texColor = textureCube( tCube, vec3( tFlip * vWorldDirection.x, vWorldDirection.yz ) );
	gl_FragColor = texColor;
	gl_FragColor.a *= opacity;
	#include <tonemapping_fragment>
	#include <colorspace_fragment>
}`,Xp=`#include <common>
#include <batching_pars_vertex>
#include <uv_pars_vertex>
#include <displacementmap_pars_vertex>
#include <morphtarget_pars_vertex>
#include <skinning_pars_vertex>
#include <logdepthbuf_pars_vertex>
#include <clipping_planes_pars_vertex>
varying vec2 vHighPrecisionZW;
void main() {
	#include <uv_vertex>
	#include <batching_vertex>
	#include <skinbase_vertex>
	#include <morphinstance_vertex>
	#ifdef USE_DISPLACEMENTMAP
		#include <beginnormal_vertex>
		#include <morphnormal_vertex>
		#include <skinnormal_vertex>
	#endif
	#include <begin_vertex>
	#include <morphtarget_vertex>
	#include <skinning_vertex>
	#include <displacementmap_vertex>
	#include <project_vertex>
	#include <logdepthbuf_vertex>
	#include <clipping_planes_vertex>
	vHighPrecisionZW = gl_Position.zw;
}`,Yp=`#if DEPTH_PACKING == 3200
	uniform float opacity;
#endif
#include <common>
#include <packing>
#include <uv_pars_fragment>
#include <map_pars_fragment>
#include <alphamap_pars_fragment>
#include <alphatest_pars_fragment>
#include <alphahash_pars_fragment>
#include <logdepthbuf_pars_fragment>
#include <clipping_planes_pars_fragment>
varying vec2 vHighPrecisionZW;
void main() {
	vec4 diffuseColor = vec4( 1.0 );
	#include <clipping_planes_fragment>
	#if DEPTH_PACKING == 3200
		diffuseColor.a = opacity;
	#endif
	#include <map_fragment>
	#include <alphamap_fragment>
	#include <alphatest_fragment>
	#include <alphahash_fragment>
	#include <logdepthbuf_fragment>
	float fragCoordZ = 0.5 * vHighPrecisionZW[0] / vHighPrecisionZW[1] + 0.5;
	#if DEPTH_PACKING == 3200
		gl_FragColor = vec4( vec3( 1.0 - fragCoordZ ), opacity );
	#elif DEPTH_PACKING == 3201
		gl_FragColor = packDepthToRGBA( fragCoordZ );
	#elif DEPTH_PACKING == 3202
		gl_FragColor = vec4( packDepthToRGB( fragCoordZ ), 1.0 );
	#elif DEPTH_PACKING == 3203
		gl_FragColor = vec4( packDepthToRG( fragCoordZ ), 0.0, 1.0 );
	#endif
}`,qp=`#define DISTANCE
varying vec3 vWorldPosition;
#include <common>
#include <batching_pars_vertex>
#include <uv_pars_vertex>
#include <displacementmap_pars_vertex>
#include <morphtarget_pars_vertex>
#include <skinning_pars_vertex>
#include <clipping_planes_pars_vertex>
void main() {
	#include <uv_vertex>
	#include <batching_vertex>
	#include <skinbase_vertex>
	#include <morphinstance_vertex>
	#ifdef USE_DISPLACEMENTMAP
		#include <beginnormal_vertex>
		#include <morphnormal_vertex>
		#include <skinnormal_vertex>
	#endif
	#include <begin_vertex>
	#include <morphtarget_vertex>
	#include <skinning_vertex>
	#include <displacementmap_vertex>
	#include <project_vertex>
	#include <worldpos_vertex>
	#include <clipping_planes_vertex>
	vWorldPosition = worldPosition.xyz;
}`,$p=`#define DISTANCE
uniform vec3 referencePosition;
uniform float nearDistance;
uniform float farDistance;
varying vec3 vWorldPosition;
#include <common>
#include <packing>
#include <uv_pars_fragment>
#include <map_pars_fragment>
#include <alphamap_pars_fragment>
#include <alphatest_pars_fragment>
#include <alphahash_pars_fragment>
#include <clipping_planes_pars_fragment>
void main () {
	vec4 diffuseColor = vec4( 1.0 );
	#include <clipping_planes_fragment>
	#include <map_fragment>
	#include <alphamap_fragment>
	#include <alphatest_fragment>
	#include <alphahash_fragment>
	float dist = length( vWorldPosition - referencePosition );
	dist = ( dist - nearDistance ) / ( farDistance - nearDistance );
	dist = saturate( dist );
	gl_FragColor = packDepthToRGBA( dist );
}`,Zp=`varying vec3 vWorldDirection;
#include <common>
void main() {
	vWorldDirection = transformDirection( position, modelMatrix );
	#include <begin_vertex>
	#include <project_vertex>
}`,Kp=`uniform sampler2D tEquirect;
varying vec3 vWorldDirection;
#include <common>
void main() {
	vec3 direction = normalize( vWorldDirection );
	vec2 sampleUV = equirectUv( direction );
	gl_FragColor = texture2D( tEquirect, sampleUV );
	#include <tonemapping_fragment>
	#include <colorspace_fragment>
}`,Jp=`uniform float scale;
attribute float lineDistance;
varying float vLineDistance;
#include <common>
#include <uv_pars_vertex>
#include <color_pars_vertex>
#include <fog_pars_vertex>
#include <morphtarget_pars_vertex>
#include <logdepthbuf_pars_vertex>
#include <clipping_planes_pars_vertex>
void main() {
	vLineDistance = scale * lineDistance;
	#include <uv_vertex>
	#include <color_vertex>
	#include <morphinstance_vertex>
	#include <morphcolor_vertex>
	#include <begin_vertex>
	#include <morphtarget_vertex>
	#include <project_vertex>
	#include <logdepthbuf_vertex>
	#include <clipping_planes_vertex>
	#include <fog_vertex>
}`,jp=`uniform vec3 diffuse;
uniform float opacity;
uniform float dashSize;
uniform float totalSize;
varying float vLineDistance;
#include <common>
#include <color_pars_fragment>
#include <uv_pars_fragment>
#include <map_pars_fragment>
#include <fog_pars_fragment>
#include <logdepthbuf_pars_fragment>
#include <clipping_planes_pars_fragment>
void main() {
	vec4 diffuseColor = vec4( diffuse, opacity );
	#include <clipping_planes_fragment>
	if ( mod( vLineDistance, totalSize ) > dashSize ) {
		discard;
	}
	vec3 outgoingLight = vec3( 0.0 );
	#include <logdepthbuf_fragment>
	#include <map_fragment>
	#include <color_fragment>
	outgoingLight = diffuseColor.rgb;
	#include <opaque_fragment>
	#include <tonemapping_fragment>
	#include <colorspace_fragment>
	#include <fog_fragment>
	#include <premultiplied_alpha_fragment>
}`,Qp=`#include <common>
#include <batching_pars_vertex>
#include <uv_pars_vertex>
#include <envmap_pars_vertex>
#include <color_pars_vertex>
#include <fog_pars_vertex>
#include <morphtarget_pars_vertex>
#include <skinning_pars_vertex>
#include <logdepthbuf_pars_vertex>
#include <clipping_planes_pars_vertex>
void main() {
	#include <uv_vertex>
	#include <color_vertex>
	#include <morphinstance_vertex>
	#include <morphcolor_vertex>
	#include <batching_vertex>
	#if defined ( USE_ENVMAP ) || defined ( USE_SKINNING )
		#include <beginnormal_vertex>
		#include <morphnormal_vertex>
		#include <skinbase_vertex>
		#include <skinnormal_vertex>
		#include <defaultnormal_vertex>
	#endif
	#include <begin_vertex>
	#include <morphtarget_vertex>
	#include <skinning_vertex>
	#include <project_vertex>
	#include <logdepthbuf_vertex>
	#include <clipping_planes_vertex>
	#include <worldpos_vertex>
	#include <envmap_vertex>
	#include <fog_vertex>
}`,tm=`uniform vec3 diffuse;
uniform float opacity;
#ifndef FLAT_SHADED
	varying vec3 vNormal;
#endif
#include <common>
#include <dithering_pars_fragment>
#include <color_pars_fragment>
#include <uv_pars_fragment>
#include <map_pars_fragment>
#include <alphamap_pars_fragment>
#include <alphatest_pars_fragment>
#include <alphahash_pars_fragment>
#include <aomap_pars_fragment>
#include <lightmap_pars_fragment>
#include <envmap_common_pars_fragment>
#include <envmap_pars_fragment>
#include <fog_pars_fragment>
#include <specularmap_pars_fragment>
#include <logdepthbuf_pars_fragment>
#include <clipping_planes_pars_fragment>
void main() {
	vec4 diffuseColor = vec4( diffuse, opacity );
	#include <clipping_planes_fragment>
	#include <logdepthbuf_fragment>
	#include <map_fragment>
	#include <color_fragment>
	#include <alphamap_fragment>
	#include <alphatest_fragment>
	#include <alphahash_fragment>
	#include <specularmap_fragment>
	ReflectedLight reflectedLight = ReflectedLight( vec3( 0.0 ), vec3( 0.0 ), vec3( 0.0 ), vec3( 0.0 ) );
	#ifdef USE_LIGHTMAP
		vec4 lightMapTexel = texture2D( lightMap, vLightMapUv );
		reflectedLight.indirectDiffuse += lightMapTexel.rgb * lightMapIntensity * RECIPROCAL_PI;
	#else
		reflectedLight.indirectDiffuse += vec3( 1.0 );
	#endif
	#include <aomap_fragment>
	reflectedLight.indirectDiffuse *= diffuseColor.rgb;
	vec3 outgoingLight = reflectedLight.indirectDiffuse;
	#include <envmap_fragment>
	#include <opaque_fragment>
	#include <tonemapping_fragment>
	#include <colorspace_fragment>
	#include <fog_fragment>
	#include <premultiplied_alpha_fragment>
	#include <dithering_fragment>
}`,em=`#define LAMBERT
varying vec3 vViewPosition;
#include <common>
#include <batching_pars_vertex>
#include <uv_pars_vertex>
#include <displacementmap_pars_vertex>
#include <envmap_pars_vertex>
#include <color_pars_vertex>
#include <fog_pars_vertex>
#include <normal_pars_vertex>
#include <morphtarget_pars_vertex>
#include <skinning_pars_vertex>
#include <shadowmap_pars_vertex>
#include <logdepthbuf_pars_vertex>
#include <clipping_planes_pars_vertex>
void main() {
	#include <uv_vertex>
	#include <color_vertex>
	#include <morphinstance_vertex>
	#include <morphcolor_vertex>
	#include <batching_vertex>
	#include <beginnormal_vertex>
	#include <morphnormal_vertex>
	#include <skinbase_vertex>
	#include <skinnormal_vertex>
	#include <defaultnormal_vertex>
	#include <normal_vertex>
	#include <begin_vertex>
	#include <morphtarget_vertex>
	#include <skinning_vertex>
	#include <displacementmap_vertex>
	#include <project_vertex>
	#include <logdepthbuf_vertex>
	#include <clipping_planes_vertex>
	vViewPosition = - mvPosition.xyz;
	#include <worldpos_vertex>
	#include <envmap_vertex>
	#include <shadowmap_vertex>
	#include <fog_vertex>
}`,nm=`#define LAMBERT
uniform vec3 diffuse;
uniform vec3 emissive;
uniform float opacity;
#include <common>
#include <packing>
#include <dithering_pars_fragment>
#include <color_pars_fragment>
#include <uv_pars_fragment>
#include <map_pars_fragment>
#include <alphamap_pars_fragment>
#include <alphatest_pars_fragment>
#include <alphahash_pars_fragment>
#include <aomap_pars_fragment>
#include <lightmap_pars_fragment>
#include <emissivemap_pars_fragment>
#include <envmap_common_pars_fragment>
#include <envmap_pars_fragment>
#include <fog_pars_fragment>
#include <bsdfs>
#include <lights_pars_begin>
#include <normal_pars_fragment>
#include <lights_lambert_pars_fragment>
#include <shadowmap_pars_fragment>
#include <bumpmap_pars_fragment>
#include <normalmap_pars_fragment>
#include <specularmap_pars_fragment>
#include <logdepthbuf_pars_fragment>
#include <clipping_planes_pars_fragment>
void main() {
	vec4 diffuseColor = vec4( diffuse, opacity );
	#include <clipping_planes_fragment>
	ReflectedLight reflectedLight = ReflectedLight( vec3( 0.0 ), vec3( 0.0 ), vec3( 0.0 ), vec3( 0.0 ) );
	vec3 totalEmissiveRadiance = emissive;
	#include <logdepthbuf_fragment>
	#include <map_fragment>
	#include <color_fragment>
	#include <alphamap_fragment>
	#include <alphatest_fragment>
	#include <alphahash_fragment>
	#include <specularmap_fragment>
	#include <normal_fragment_begin>
	#include <normal_fragment_maps>
	#include <emissivemap_fragment>
	#include <lights_lambert_fragment>
	#include <lights_fragment_begin>
	#include <lights_fragment_maps>
	#include <lights_fragment_end>
	#include <aomap_fragment>
	vec3 outgoingLight = reflectedLight.directDiffuse + reflectedLight.indirectDiffuse + totalEmissiveRadiance;
	#include <envmap_fragment>
	#include <opaque_fragment>
	#include <tonemapping_fragment>
	#include <colorspace_fragment>
	#include <fog_fragment>
	#include <premultiplied_alpha_fragment>
	#include <dithering_fragment>
}`,im=`#define MATCAP
varying vec3 vViewPosition;
#include <common>
#include <batching_pars_vertex>
#include <uv_pars_vertex>
#include <color_pars_vertex>
#include <displacementmap_pars_vertex>
#include <fog_pars_vertex>
#include <normal_pars_vertex>
#include <morphtarget_pars_vertex>
#include <skinning_pars_vertex>
#include <logdepthbuf_pars_vertex>
#include <clipping_planes_pars_vertex>
void main() {
	#include <uv_vertex>
	#include <color_vertex>
	#include <morphinstance_vertex>
	#include <morphcolor_vertex>
	#include <batching_vertex>
	#include <beginnormal_vertex>
	#include <morphnormal_vertex>
	#include <skinbase_vertex>
	#include <skinnormal_vertex>
	#include <defaultnormal_vertex>
	#include <normal_vertex>
	#include <begin_vertex>
	#include <morphtarget_vertex>
	#include <skinning_vertex>
	#include <displacementmap_vertex>
	#include <project_vertex>
	#include <logdepthbuf_vertex>
	#include <clipping_planes_vertex>
	#include <fog_vertex>
	vViewPosition = - mvPosition.xyz;
}`,sm=`#define MATCAP
uniform vec3 diffuse;
uniform float opacity;
uniform sampler2D matcap;
varying vec3 vViewPosition;
#include <common>
#include <dithering_pars_fragment>
#include <color_pars_fragment>
#include <uv_pars_fragment>
#include <map_pars_fragment>
#include <alphamap_pars_fragment>
#include <alphatest_pars_fragment>
#include <alphahash_pars_fragment>
#include <fog_pars_fragment>
#include <normal_pars_fragment>
#include <bumpmap_pars_fragment>
#include <normalmap_pars_fragment>
#include <logdepthbuf_pars_fragment>
#include <clipping_planes_pars_fragment>
void main() {
	vec4 diffuseColor = vec4( diffuse, opacity );
	#include <clipping_planes_fragment>
	#include <logdepthbuf_fragment>
	#include <map_fragment>
	#include <color_fragment>
	#include <alphamap_fragment>
	#include <alphatest_fragment>
	#include <alphahash_fragment>
	#include <normal_fragment_begin>
	#include <normal_fragment_maps>
	vec3 viewDir = normalize( vViewPosition );
	vec3 x = normalize( vec3( viewDir.z, 0.0, - viewDir.x ) );
	vec3 y = cross( viewDir, x );
	vec2 uv = vec2( dot( x, normal ), dot( y, normal ) ) * 0.495 + 0.5;
	#ifdef USE_MATCAP
		vec4 matcapColor = texture2D( matcap, uv );
	#else
		vec4 matcapColor = vec4( vec3( mix( 0.2, 0.8, uv.y ) ), 1.0 );
	#endif
	vec3 outgoingLight = diffuseColor.rgb * matcapColor.rgb;
	#include <opaque_fragment>
	#include <tonemapping_fragment>
	#include <colorspace_fragment>
	#include <fog_fragment>
	#include <premultiplied_alpha_fragment>
	#include <dithering_fragment>
}`,rm=`#define NORMAL
#if defined( FLAT_SHADED ) || defined( USE_BUMPMAP ) || defined( USE_NORMALMAP_TANGENTSPACE )
	varying vec3 vViewPosition;
#endif
#include <common>
#include <batching_pars_vertex>
#include <uv_pars_vertex>
#include <displacementmap_pars_vertex>
#include <normal_pars_vertex>
#include <morphtarget_pars_vertex>
#include <skinning_pars_vertex>
#include <logdepthbuf_pars_vertex>
#include <clipping_planes_pars_vertex>
void main() {
	#include <uv_vertex>
	#include <batching_vertex>
	#include <beginnormal_vertex>
	#include <morphinstance_vertex>
	#include <morphnormal_vertex>
	#include <skinbase_vertex>
	#include <skinnormal_vertex>
	#include <defaultnormal_vertex>
	#include <normal_vertex>
	#include <begin_vertex>
	#include <morphtarget_vertex>
	#include <skinning_vertex>
	#include <displacementmap_vertex>
	#include <project_vertex>
	#include <logdepthbuf_vertex>
	#include <clipping_planes_vertex>
#if defined( FLAT_SHADED ) || defined( USE_BUMPMAP ) || defined( USE_NORMALMAP_TANGENTSPACE )
	vViewPosition = - mvPosition.xyz;
#endif
}`,om=`#define NORMAL
uniform float opacity;
#if defined( FLAT_SHADED ) || defined( USE_BUMPMAP ) || defined( USE_NORMALMAP_TANGENTSPACE )
	varying vec3 vViewPosition;
#endif
#include <packing>
#include <uv_pars_fragment>
#include <normal_pars_fragment>
#include <bumpmap_pars_fragment>
#include <normalmap_pars_fragment>
#include <logdepthbuf_pars_fragment>
#include <clipping_planes_pars_fragment>
void main() {
	vec4 diffuseColor = vec4( 0.0, 0.0, 0.0, opacity );
	#include <clipping_planes_fragment>
	#include <logdepthbuf_fragment>
	#include <normal_fragment_begin>
	#include <normal_fragment_maps>
	gl_FragColor = vec4( packNormalToRGB( normal ), diffuseColor.a );
	#ifdef OPAQUE
		gl_FragColor.a = 1.0;
	#endif
}`,am=`#define PHONG
varying vec3 vViewPosition;
#include <common>
#include <batching_pars_vertex>
#include <uv_pars_vertex>
#include <displacementmap_pars_vertex>
#include <envmap_pars_vertex>
#include <color_pars_vertex>
#include <fog_pars_vertex>
#include <normal_pars_vertex>
#include <morphtarget_pars_vertex>
#include <skinning_pars_vertex>
#include <shadowmap_pars_vertex>
#include <logdepthbuf_pars_vertex>
#include <clipping_planes_pars_vertex>
void main() {
	#include <uv_vertex>
	#include <color_vertex>
	#include <morphcolor_vertex>
	#include <batching_vertex>
	#include <beginnormal_vertex>
	#include <morphinstance_vertex>
	#include <morphnormal_vertex>
	#include <skinbase_vertex>
	#include <skinnormal_vertex>
	#include <defaultnormal_vertex>
	#include <normal_vertex>
	#include <begin_vertex>
	#include <morphtarget_vertex>
	#include <skinning_vertex>
	#include <displacementmap_vertex>
	#include <project_vertex>
	#include <logdepthbuf_vertex>
	#include <clipping_planes_vertex>
	vViewPosition = - mvPosition.xyz;
	#include <worldpos_vertex>
	#include <envmap_vertex>
	#include <shadowmap_vertex>
	#include <fog_vertex>
}`,cm=`#define PHONG
uniform vec3 diffuse;
uniform vec3 emissive;
uniform vec3 specular;
uniform float shininess;
uniform float opacity;
#include <common>
#include <packing>
#include <dithering_pars_fragment>
#include <color_pars_fragment>
#include <uv_pars_fragment>
#include <map_pars_fragment>
#include <alphamap_pars_fragment>
#include <alphatest_pars_fragment>
#include <alphahash_pars_fragment>
#include <aomap_pars_fragment>
#include <lightmap_pars_fragment>
#include <emissivemap_pars_fragment>
#include <envmap_common_pars_fragment>
#include <envmap_pars_fragment>
#include <fog_pars_fragment>
#include <bsdfs>
#include <lights_pars_begin>
#include <normal_pars_fragment>
#include <lights_phong_pars_fragment>
#include <shadowmap_pars_fragment>
#include <bumpmap_pars_fragment>
#include <normalmap_pars_fragment>
#include <specularmap_pars_fragment>
#include <logdepthbuf_pars_fragment>
#include <clipping_planes_pars_fragment>
void main() {
	vec4 diffuseColor = vec4( diffuse, opacity );
	#include <clipping_planes_fragment>
	ReflectedLight reflectedLight = ReflectedLight( vec3( 0.0 ), vec3( 0.0 ), vec3( 0.0 ), vec3( 0.0 ) );
	vec3 totalEmissiveRadiance = emissive;
	#include <logdepthbuf_fragment>
	#include <map_fragment>
	#include <color_fragment>
	#include <alphamap_fragment>
	#include <alphatest_fragment>
	#include <alphahash_fragment>
	#include <specularmap_fragment>
	#include <normal_fragment_begin>
	#include <normal_fragment_maps>
	#include <emissivemap_fragment>
	#include <lights_phong_fragment>
	#include <lights_fragment_begin>
	#include <lights_fragment_maps>
	#include <lights_fragment_end>
	#include <aomap_fragment>
	vec3 outgoingLight = reflectedLight.directDiffuse + reflectedLight.indirectDiffuse + reflectedLight.directSpecular + reflectedLight.indirectSpecular + totalEmissiveRadiance;
	#include <envmap_fragment>
	#include <opaque_fragment>
	#include <tonemapping_fragment>
	#include <colorspace_fragment>
	#include <fog_fragment>
	#include <premultiplied_alpha_fragment>
	#include <dithering_fragment>
}`,lm=`#define STANDARD
varying vec3 vViewPosition;
#ifdef USE_TRANSMISSION
	varying vec3 vWorldPosition;
#endif
#include <common>
#include <batching_pars_vertex>
#include <uv_pars_vertex>
#include <displacementmap_pars_vertex>
#include <color_pars_vertex>
#include <fog_pars_vertex>
#include <normal_pars_vertex>
#include <morphtarget_pars_vertex>
#include <skinning_pars_vertex>
#include <shadowmap_pars_vertex>
#include <logdepthbuf_pars_vertex>
#include <clipping_planes_pars_vertex>
void main() {
	#include <uv_vertex>
	#include <color_vertex>
	#include <morphinstance_vertex>
	#include <morphcolor_vertex>
	#include <batching_vertex>
	#include <beginnormal_vertex>
	#include <morphnormal_vertex>
	#include <skinbase_vertex>
	#include <skinnormal_vertex>
	#include <defaultnormal_vertex>
	#include <normal_vertex>
	#include <begin_vertex>
	#include <morphtarget_vertex>
	#include <skinning_vertex>
	#include <displacementmap_vertex>
	#include <project_vertex>
	#include <logdepthbuf_vertex>
	#include <clipping_planes_vertex>
	vViewPosition = - mvPosition.xyz;
	#include <worldpos_vertex>
	#include <shadowmap_vertex>
	#include <fog_vertex>
#ifdef USE_TRANSMISSION
	vWorldPosition = worldPosition.xyz;
#endif
}`,hm=`#define STANDARD
#ifdef PHYSICAL
	#define IOR
	#define USE_SPECULAR
#endif
uniform vec3 diffuse;
uniform vec3 emissive;
uniform float roughness;
uniform float metalness;
uniform float opacity;
#ifdef IOR
	uniform float ior;
#endif
#ifdef USE_SPECULAR
	uniform float specularIntensity;
	uniform vec3 specularColor;
	#ifdef USE_SPECULAR_COLORMAP
		uniform sampler2D specularColorMap;
	#endif
	#ifdef USE_SPECULAR_INTENSITYMAP
		uniform sampler2D specularIntensityMap;
	#endif
#endif
#ifdef USE_CLEARCOAT
	uniform float clearcoat;
	uniform float clearcoatRoughness;
#endif
#ifdef USE_DISPERSION
	uniform float dispersion;
#endif
#ifdef USE_IRIDESCENCE
	uniform float iridescence;
	uniform float iridescenceIOR;
	uniform float iridescenceThicknessMinimum;
	uniform float iridescenceThicknessMaximum;
#endif
#ifdef USE_SHEEN
	uniform vec3 sheenColor;
	uniform float sheenRoughness;
	#ifdef USE_SHEEN_COLORMAP
		uniform sampler2D sheenColorMap;
	#endif
	#ifdef USE_SHEEN_ROUGHNESSMAP
		uniform sampler2D sheenRoughnessMap;
	#endif
#endif
#ifdef USE_ANISOTROPY
	uniform vec2 anisotropyVector;
	#ifdef USE_ANISOTROPYMAP
		uniform sampler2D anisotropyMap;
	#endif
#endif
varying vec3 vViewPosition;
#include <common>
#include <packing>
#include <dithering_pars_fragment>
#include <color_pars_fragment>
#include <uv_pars_fragment>
#include <map_pars_fragment>
#include <alphamap_pars_fragment>
#include <alphatest_pars_fragment>
#include <alphahash_pars_fragment>
#include <aomap_pars_fragment>
#include <lightmap_pars_fragment>
#include <emissivemap_pars_fragment>
#include <iridescence_fragment>
#include <cube_uv_reflection_fragment>
#include <envmap_common_pars_fragment>
#include <envmap_physical_pars_fragment>
#include <fog_pars_fragment>
#include <lights_pars_begin>
#include <normal_pars_fragment>
#include <lights_physical_pars_fragment>
#include <transmission_pars_fragment>
#include <shadowmap_pars_fragment>
#include <bumpmap_pars_fragment>
#include <normalmap_pars_fragment>
#include <clearcoat_pars_fragment>
#include <iridescence_pars_fragment>
#include <roughnessmap_pars_fragment>
#include <metalnessmap_pars_fragment>
#include <logdepthbuf_pars_fragment>
#include <clipping_planes_pars_fragment>
void main() {
	vec4 diffuseColor = vec4( diffuse, opacity );
	#include <clipping_planes_fragment>
	ReflectedLight reflectedLight = ReflectedLight( vec3( 0.0 ), vec3( 0.0 ), vec3( 0.0 ), vec3( 0.0 ) );
	vec3 totalEmissiveRadiance = emissive;
	#include <logdepthbuf_fragment>
	#include <map_fragment>
	#include <color_fragment>
	#include <alphamap_fragment>
	#include <alphatest_fragment>
	#include <alphahash_fragment>
	#include <roughnessmap_fragment>
	#include <metalnessmap_fragment>
	#include <normal_fragment_begin>
	#include <normal_fragment_maps>
	#include <clearcoat_normal_fragment_begin>
	#include <clearcoat_normal_fragment_maps>
	#include <emissivemap_fragment>
	#include <lights_physical_fragment>
	#include <lights_fragment_begin>
	#include <lights_fragment_maps>
	#include <lights_fragment_end>
	#include <aomap_fragment>
	vec3 totalDiffuse = reflectedLight.directDiffuse + reflectedLight.indirectDiffuse;
	vec3 totalSpecular = reflectedLight.directSpecular + reflectedLight.indirectSpecular;
	#include <transmission_fragment>
	vec3 outgoingLight = totalDiffuse + totalSpecular + totalEmissiveRadiance;
	#ifdef USE_SHEEN
		float sheenEnergyComp = 1.0 - 0.157 * max3( material.sheenColor );
		outgoingLight = outgoingLight * sheenEnergyComp + sheenSpecularDirect + sheenSpecularIndirect;
	#endif
	#ifdef USE_CLEARCOAT
		float dotNVcc = saturate( dot( geometryClearcoatNormal, geometryViewDir ) );
		vec3 Fcc = F_Schlick( material.clearcoatF0, material.clearcoatF90, dotNVcc );
		outgoingLight = outgoingLight * ( 1.0 - material.clearcoat * Fcc ) + ( clearcoatSpecularDirect + clearcoatSpecularIndirect ) * material.clearcoat;
	#endif
	#include <opaque_fragment>
	#include <tonemapping_fragment>
	#include <colorspace_fragment>
	#include <fog_fragment>
	#include <premultiplied_alpha_fragment>
	#include <dithering_fragment>
}`,um=`#define TOON
varying vec3 vViewPosition;
#include <common>
#include <batching_pars_vertex>
#include <uv_pars_vertex>
#include <displacementmap_pars_vertex>
#include <color_pars_vertex>
#include <fog_pars_vertex>
#include <normal_pars_vertex>
#include <morphtarget_pars_vertex>
#include <skinning_pars_vertex>
#include <shadowmap_pars_vertex>
#include <logdepthbuf_pars_vertex>
#include <clipping_planes_pars_vertex>
void main() {
	#include <uv_vertex>
	#include <color_vertex>
	#include <morphinstance_vertex>
	#include <morphcolor_vertex>
	#include <batching_vertex>
	#include <beginnormal_vertex>
	#include <morphnormal_vertex>
	#include <skinbase_vertex>
	#include <skinnormal_vertex>
	#include <defaultnormal_vertex>
	#include <normal_vertex>
	#include <begin_vertex>
	#include <morphtarget_vertex>
	#include <skinning_vertex>
	#include <displacementmap_vertex>
	#include <project_vertex>
	#include <logdepthbuf_vertex>
	#include <clipping_planes_vertex>
	vViewPosition = - mvPosition.xyz;
	#include <worldpos_vertex>
	#include <shadowmap_vertex>
	#include <fog_vertex>
}`,dm=`#define TOON
uniform vec3 diffuse;
uniform vec3 emissive;
uniform float opacity;
#include <common>
#include <packing>
#include <dithering_pars_fragment>
#include <color_pars_fragment>
#include <uv_pars_fragment>
#include <map_pars_fragment>
#include <alphamap_pars_fragment>
#include <alphatest_pars_fragment>
#include <alphahash_pars_fragment>
#include <aomap_pars_fragment>
#include <lightmap_pars_fragment>
#include <emissivemap_pars_fragment>
#include <gradientmap_pars_fragment>
#include <fog_pars_fragment>
#include <bsdfs>
#include <lights_pars_begin>
#include <normal_pars_fragment>
#include <lights_toon_pars_fragment>
#include <shadowmap_pars_fragment>
#include <bumpmap_pars_fragment>
#include <normalmap_pars_fragment>
#include <logdepthbuf_pars_fragment>
#include <clipping_planes_pars_fragment>
void main() {
	vec4 diffuseColor = vec4( diffuse, opacity );
	#include <clipping_planes_fragment>
	ReflectedLight reflectedLight = ReflectedLight( vec3( 0.0 ), vec3( 0.0 ), vec3( 0.0 ), vec3( 0.0 ) );
	vec3 totalEmissiveRadiance = emissive;
	#include <logdepthbuf_fragment>
	#include <map_fragment>
	#include <color_fragment>
	#include <alphamap_fragment>
	#include <alphatest_fragment>
	#include <alphahash_fragment>
	#include <normal_fragment_begin>
	#include <normal_fragment_maps>
	#include <emissivemap_fragment>
	#include <lights_toon_fragment>
	#include <lights_fragment_begin>
	#include <lights_fragment_maps>
	#include <lights_fragment_end>
	#include <aomap_fragment>
	vec3 outgoingLight = reflectedLight.directDiffuse + reflectedLight.indirectDiffuse + totalEmissiveRadiance;
	#include <opaque_fragment>
	#include <tonemapping_fragment>
	#include <colorspace_fragment>
	#include <fog_fragment>
	#include <premultiplied_alpha_fragment>
	#include <dithering_fragment>
}`,fm=`uniform float size;
uniform float scale;
#include <common>
#include <color_pars_vertex>
#include <fog_pars_vertex>
#include <morphtarget_pars_vertex>
#include <logdepthbuf_pars_vertex>
#include <clipping_planes_pars_vertex>
#ifdef USE_POINTS_UV
	varying vec2 vUv;
	uniform mat3 uvTransform;
#endif
void main() {
	#ifdef USE_POINTS_UV
		vUv = ( uvTransform * vec3( uv, 1 ) ).xy;
	#endif
	#include <color_vertex>
	#include <morphinstance_vertex>
	#include <morphcolor_vertex>
	#include <begin_vertex>
	#include <morphtarget_vertex>
	#include <project_vertex>
	gl_PointSize = size;
	#ifdef USE_SIZEATTENUATION
		bool isPerspective = isPerspectiveMatrix( projectionMatrix );
		if ( isPerspective ) gl_PointSize *= ( scale / - mvPosition.z );
	#endif
	#include <logdepthbuf_vertex>
	#include <clipping_planes_vertex>
	#include <worldpos_vertex>
	#include <fog_vertex>
}`,pm=`uniform vec3 diffuse;
uniform float opacity;
#include <common>
#include <color_pars_fragment>
#include <map_particle_pars_fragment>
#include <alphatest_pars_fragment>
#include <alphahash_pars_fragment>
#include <fog_pars_fragment>
#include <logdepthbuf_pars_fragment>
#include <clipping_planes_pars_fragment>
void main() {
	vec4 diffuseColor = vec4( diffuse, opacity );
	#include <clipping_planes_fragment>
	vec3 outgoingLight = vec3( 0.0 );
	#include <logdepthbuf_fragment>
	#include <map_particle_fragment>
	#include <color_fragment>
	#include <alphatest_fragment>
	#include <alphahash_fragment>
	outgoingLight = diffuseColor.rgb;
	#include <opaque_fragment>
	#include <tonemapping_fragment>
	#include <colorspace_fragment>
	#include <fog_fragment>
	#include <premultiplied_alpha_fragment>
}`,mm=`#include <common>
#include <batching_pars_vertex>
#include <fog_pars_vertex>
#include <morphtarget_pars_vertex>
#include <skinning_pars_vertex>
#include <logdepthbuf_pars_vertex>
#include <shadowmap_pars_vertex>
void main() {
	#include <batching_vertex>
	#include <beginnormal_vertex>
	#include <morphinstance_vertex>
	#include <morphnormal_vertex>
	#include <skinbase_vertex>
	#include <skinnormal_vertex>
	#include <defaultnormal_vertex>
	#include <begin_vertex>
	#include <morphtarget_vertex>
	#include <skinning_vertex>
	#include <project_vertex>
	#include <logdepthbuf_vertex>
	#include <worldpos_vertex>
	#include <shadowmap_vertex>
	#include <fog_vertex>
}`,gm=`uniform vec3 color;
uniform float opacity;
#include <common>
#include <packing>
#include <fog_pars_fragment>
#include <bsdfs>
#include <lights_pars_begin>
#include <logdepthbuf_pars_fragment>
#include <shadowmap_pars_fragment>
#include <shadowmask_pars_fragment>
void main() {
	#include <logdepthbuf_fragment>
	gl_FragColor = vec4( color, opacity * ( 1.0 - getShadowMask() ) );
	#include <tonemapping_fragment>
	#include <colorspace_fragment>
	#include <fog_fragment>
}`,_m=`uniform float rotation;
uniform vec2 center;
#include <common>
#include <uv_pars_vertex>
#include <fog_pars_vertex>
#include <logdepthbuf_pars_vertex>
#include <clipping_planes_pars_vertex>
void main() {
	#include <uv_vertex>
	vec4 mvPosition = modelViewMatrix[ 3 ];
	vec2 scale = vec2( length( modelMatrix[ 0 ].xyz ), length( modelMatrix[ 1 ].xyz ) );
	#ifndef USE_SIZEATTENUATION
		bool isPerspective = isPerspectiveMatrix( projectionMatrix );
		if ( isPerspective ) scale *= - mvPosition.z;
	#endif
	vec2 alignedPosition = ( position.xy - ( center - vec2( 0.5 ) ) ) * scale;
	vec2 rotatedPosition;
	rotatedPosition.x = cos( rotation ) * alignedPosition.x - sin( rotation ) * alignedPosition.y;
	rotatedPosition.y = sin( rotation ) * alignedPosition.x + cos( rotation ) * alignedPosition.y;
	mvPosition.xy += rotatedPosition;
	gl_Position = projectionMatrix * mvPosition;
	#include <logdepthbuf_vertex>
	#include <clipping_planes_vertex>
	#include <fog_vertex>
}`,xm=`uniform vec3 diffuse;
uniform float opacity;
#include <common>
#include <uv_pars_fragment>
#include <map_pars_fragment>
#include <alphamap_pars_fragment>
#include <alphatest_pars_fragment>
#include <alphahash_pars_fragment>
#include <fog_pars_fragment>
#include <logdepthbuf_pars_fragment>
#include <clipping_planes_pars_fragment>
void main() {
	vec4 diffuseColor = vec4( diffuse, opacity );
	#include <clipping_planes_fragment>
	vec3 outgoingLight = vec3( 0.0 );
	#include <logdepthbuf_fragment>
	#include <map_fragment>
	#include <alphamap_fragment>
	#include <alphatest_fragment>
	#include <alphahash_fragment>
	outgoingLight = diffuseColor.rgb;
	#include <opaque_fragment>
	#include <tonemapping_fragment>
	#include <colorspace_fragment>
	#include <fog_fragment>
}`,ie={alphahash_fragment:zd,alphahash_pars_fragment:kd,alphamap_fragment:Hd,alphamap_pars_fragment:Vd,alphatest_fragment:Gd,alphatest_pars_fragment:Wd,aomap_fragment:Xd,aomap_pars_fragment:Yd,batching_pars_vertex:qd,batching_vertex:$d,begin_vertex:Zd,beginnormal_vertex:Kd,bsdfs:Jd,iridescence_fragment:jd,bumpmap_pars_fragment:Qd,clipping_planes_fragment:tf,clipping_planes_pars_fragment:ef,clipping_planes_pars_vertex:nf,clipping_planes_vertex:sf,color_fragment:rf,color_pars_fragment:of,color_pars_vertex:af,color_vertex:cf,common:lf,cube_uv_reflection_fragment:hf,defaultnormal_vertex:uf,displacementmap_pars_vertex:df,displacementmap_vertex:ff,emissivemap_fragment:pf,emissivemap_pars_fragment:mf,colorspace_fragment:gf,colorspace_pars_fragment:_f,envmap_fragment:xf,envmap_common_pars_fragment:yf,envmap_pars_fragment:vf,envmap_pars_vertex:Mf,envmap_physical_pars_fragment:Lf,envmap_vertex:Sf,fog_vertex:bf,fog_pars_vertex:Ef,fog_fragment:Tf,fog_pars_fragment:wf,gradientmap_pars_fragment:Af,lightmap_pars_fragment:Cf,lights_lambert_fragment:Rf,lights_lambert_pars_fragment:Pf,lights_pars_begin:If,lights_toon_fragment:Df,lights_toon_pars_fragment:Uf,lights_phong_fragment:Nf,lights_phong_pars_fragment:Ff,lights_physical_fragment:Of,lights_physical_pars_fragment:Bf,lights_fragment_begin:zf,lights_fragment_maps:kf,lights_fragment_end:Hf,logdepthbuf_fragment:Vf,logdepthbuf_pars_fragment:Gf,logdepthbuf_pars_vertex:Wf,logdepthbuf_vertex:Xf,map_fragment:Yf,map_pars_fragment:qf,map_particle_fragment:$f,map_particle_pars_fragment:Zf,metalnessmap_fragment:Kf,metalnessmap_pars_fragment:Jf,morphinstance_vertex:jf,morphcolor_vertex:Qf,morphnormal_vertex:tp,morphtarget_pars_vertex:ep,morphtarget_vertex:np,normal_fragment_begin:ip,normal_fragment_maps:sp,normal_pars_fragment:rp,normal_pars_vertex:op,normal_vertex:ap,normalmap_pars_fragment:cp,clearcoat_normal_fragment_begin:lp,clearcoat_normal_fragment_maps:hp,clearcoat_pars_fragment:up,iridescence_pars_fragment:dp,opaque_fragment:fp,packing:pp,premultiplied_alpha_fragment:mp,project_vertex:gp,dithering_fragment:_p,dithering_pars_fragment:xp,roughnessmap_fragment:yp,roughnessmap_pars_fragment:vp,shadowmap_pars_fragment:Mp,shadowmap_pars_vertex:Sp,shadowmap_vertex:bp,shadowmask_pars_fragment:Ep,skinbase_vertex:Tp,skinning_pars_vertex:wp,skinning_vertex:Ap,skinnormal_vertex:Cp,specularmap_fragment:Rp,specularmap_pars_fragment:Pp,tonemapping_fragment:Ip,tonemapping_pars_fragment:Lp,transmission_fragment:Dp,transmission_pars_fragment:Up,uv_pars_fragment:Np,uv_pars_vertex:Fp,uv_vertex:Op,worldpos_vertex:Bp,background_vert:zp,background_frag:kp,backgroundCube_vert:Hp,backgroundCube_frag:Vp,cube_vert:Gp,cube_frag:Wp,depth_vert:Xp,depth_frag:Yp,distanceRGBA_vert:qp,distanceRGBA_frag:$p,equirect_vert:Zp,equirect_frag:Kp,linedashed_vert:Jp,linedashed_frag:jp,meshbasic_vert:Qp,meshbasic_frag:tm,meshlambert_vert:em,meshlambert_frag:nm,meshmatcap_vert:im,meshmatcap_frag:sm,meshnormal_vert:rm,meshnormal_frag:om,meshphong_vert:am,meshphong_frag:cm,meshphysical_vert:lm,meshphysical_frag:hm,meshtoon_vert:um,meshtoon_frag:dm,points_vert:fm,points_frag:pm,shadow_vert:mm,shadow_frag:gm,sprite_vert:_m,sprite_frag:xm},Mt={common:{diffuse:{value:new _t(16777215)},opacity:{value:1},map:{value:null},mapTransform:{value:new ee},alphaMap:{value:null},alphaMapTransform:{value:new ee},alphaTest:{value:0}},specularmap:{specularMap:{value:null},specularMapTransform:{value:new ee}},envmap:{envMap:{value:null},envMapRotation:{value:new ee},flipEnvMap:{value:-1},reflectivity:{value:1},ior:{value:1.5},refractionRatio:{value:.98}},aomap:{aoMap:{value:null},aoMapIntensity:{value:1},aoMapTransform:{value:new ee}},lightmap:{lightMap:{value:null},lightMapIntensity:{value:1},lightMapTransform:{value:new ee}},bumpmap:{bumpMap:{value:null},bumpMapTransform:{value:new ee},bumpScale:{value:1}},normalmap:{normalMap:{value:null},normalMapTransform:{value:new ee},normalScale:{value:new Wt(1,1)}},displacementmap:{displacementMap:{value:null},displacementMapTransform:{value:new ee},displacementScale:{value:1},displacementBias:{value:0}},emissivemap:{emissiveMap:{value:null},emissiveMapTransform:{value:new ee}},metalnessmap:{metalnessMap:{value:null},metalnessMapTransform:{value:new ee}},roughnessmap:{roughnessMap:{value:null},roughnessMapTransform:{value:new ee}},gradientmap:{gradientMap:{value:null}},fog:{fogDensity:{value:25e-5},fogNear:{value:1},fogFar:{value:2e3},fogColor:{value:new _t(16777215)}},lights:{ambientLightColor:{value:[]},lightProbe:{value:[]},directionalLights:{value:[],properties:{direction:{},color:{}}},directionalLightShadows:{value:[],properties:{shadowIntensity:1,shadowBias:{},shadowNormalBias:{},shadowRadius:{},shadowMapSize:{}}},directionalShadowMap:{value:[]},directionalShadowMatrix:{value:[]},spotLights:{value:[],properties:{color:{},position:{},direction:{},distance:{},coneCos:{},penumbraCos:{},decay:{}}},spotLightShadows:{value:[],properties:{shadowIntensity:1,shadowBias:{},shadowNormalBias:{},shadowRadius:{},shadowMapSize:{}}},spotLightMap:{value:[]},spotShadowMap:{value:[]},spotLightMatrix:{value:[]},pointLights:{value:[],properties:{color:{},position:{},decay:{},distance:{}}},pointLightShadows:{value:[],properties:{shadowIntensity:1,shadowBias:{},shadowNormalBias:{},shadowRadius:{},shadowMapSize:{},shadowCameraNear:{},shadowCameraFar:{}}},pointShadowMap:{value:[]},pointShadowMatrix:{value:[]},hemisphereLights:{value:[],properties:{direction:{},skyColor:{},groundColor:{}}},rectAreaLights:{value:[],properties:{color:{},position:{},width:{},height:{}}},ltc_1:{value:null},ltc_2:{value:null}},points:{diffuse:{value:new _t(16777215)},opacity:{value:1},size:{value:1},scale:{value:1},map:{value:null},alphaMap:{value:null},alphaMapTransform:{value:new ee},alphaTest:{value:0},uvTransform:{value:new ee}},sprite:{diffuse:{value:new _t(16777215)},opacity:{value:1},center:{value:new Wt(.5,.5)},rotation:{value:0},map:{value:null},mapTransform:{value:new ee},alphaMap:{value:null},alphaMapTransform:{value:new ee},alphaTest:{value:0}}},In={basic:{uniforms:Je([Mt.common,Mt.specularmap,Mt.envmap,Mt.aomap,Mt.lightmap,Mt.fog]),vertexShader:ie.meshbasic_vert,fragmentShader:ie.meshbasic_frag},lambert:{uniforms:Je([Mt.common,Mt.specularmap,Mt.envmap,Mt.aomap,Mt.lightmap,Mt.emissivemap,Mt.bumpmap,Mt.normalmap,Mt.displacementmap,Mt.fog,Mt.lights,{emissive:{value:new _t(0)}}]),vertexShader:ie.meshlambert_vert,fragmentShader:ie.meshlambert_frag},phong:{uniforms:Je([Mt.common,Mt.specularmap,Mt.envmap,Mt.aomap,Mt.lightmap,Mt.emissivemap,Mt.bumpmap,Mt.normalmap,Mt.displacementmap,Mt.fog,Mt.lights,{emissive:{value:new _t(0)},specular:{value:new _t(1118481)},shininess:{value:30}}]),vertexShader:ie.meshphong_vert,fragmentShader:ie.meshphong_frag},standard:{uniforms:Je([Mt.common,Mt.envmap,Mt.aomap,Mt.lightmap,Mt.emissivemap,Mt.bumpmap,Mt.normalmap,Mt.displacementmap,Mt.roughnessmap,Mt.metalnessmap,Mt.fog,Mt.lights,{emissive:{value:new _t(0)},roughness:{value:1},metalness:{value:0},envMapIntensity:{value:1}}]),vertexShader:ie.meshphysical_vert,fragmentShader:ie.meshphysical_frag},toon:{uniforms:Je([Mt.common,Mt.aomap,Mt.lightmap,Mt.emissivemap,Mt.bumpmap,Mt.normalmap,Mt.displacementmap,Mt.gradientmap,Mt.fog,Mt.lights,{emissive:{value:new _t(0)}}]),vertexShader:ie.meshtoon_vert,fragmentShader:ie.meshtoon_frag},matcap:{uniforms:Je([Mt.common,Mt.bumpmap,Mt.normalmap,Mt.displacementmap,Mt.fog,{matcap:{value:null}}]),vertexShader:ie.meshmatcap_vert,fragmentShader:ie.meshmatcap_frag},points:{uniforms:Je([Mt.points,Mt.fog]),vertexShader:ie.points_vert,fragmentShader:ie.points_frag},dashed:{uniforms:Je([Mt.common,Mt.fog,{scale:{value:1},dashSize:{value:1},totalSize:{value:2}}]),vertexShader:ie.linedashed_vert,fragmentShader:ie.linedashed_frag},depth:{uniforms:Je([Mt.common,Mt.displacementmap]),vertexShader:ie.depth_vert,fragmentShader:ie.depth_frag},normal:{uniforms:Je([Mt.common,Mt.bumpmap,Mt.normalmap,Mt.displacementmap,{opacity:{value:1}}]),vertexShader:ie.meshnormal_vert,fragmentShader:ie.meshnormal_frag},sprite:{uniforms:Je([Mt.sprite,Mt.fog]),vertexShader:ie.sprite_vert,fragmentShader:ie.sprite_frag},background:{uniforms:{uvTransform:{value:new ee},t2D:{value:null},backgroundIntensity:{value:1}},vertexShader:ie.background_vert,fragmentShader:ie.background_frag},backgroundCube:{uniforms:{envMap:{value:null},flipEnvMap:{value:-1},backgroundBlurriness:{value:0},backgroundIntensity:{value:1},backgroundRotation:{value:new ee}},vertexShader:ie.backgroundCube_vert,fragmentShader:ie.backgroundCube_frag},cube:{uniforms:{tCube:{value:null},tFlip:{value:-1},opacity:{value:1}},vertexShader:ie.cube_vert,fragmentShader:ie.cube_frag},equirect:{uniforms:{tEquirect:{value:null}},vertexShader:ie.equirect_vert,fragmentShader:ie.equirect_frag},distanceRGBA:{uniforms:Je([Mt.common,Mt.displacementmap,{referencePosition:{value:new U},nearDistance:{value:1},farDistance:{value:1e3}}]),vertexShader:ie.distanceRGBA_vert,fragmentShader:ie.distanceRGBA_frag},shadow:{uniforms:Je([Mt.lights,Mt.fog,{color:{value:new _t(0)},opacity:{value:1}}]),vertexShader:ie.shadow_vert,fragmentShader:ie.shadow_frag}};In.physical={uniforms:Je([In.standard.uniforms,{clearcoat:{value:0},clearcoatMap:{value:null},clearcoatMapTransform:{value:new ee},clearcoatNormalMap:{value:null},clearcoatNormalMapTransform:{value:new ee},clearcoatNormalScale:{value:new Wt(1,1)},clearcoatRoughness:{value:0},clearcoatRoughnessMap:{value:null},clearcoatRoughnessMapTransform:{value:new ee},dispersion:{value:0},iridescence:{value:0},iridescenceMap:{value:null},iridescenceMapTransform:{value:new ee},iridescenceIOR:{value:1.3},iridescenceThicknessMinimum:{value:100},iridescenceThicknessMaximum:{value:400},iridescenceThicknessMap:{value:null},iridescenceThicknessMapTransform:{value:new ee},sheen:{value:0},sheenColor:{value:new _t(0)},sheenColorMap:{value:null},sheenColorMapTransform:{value:new ee},sheenRoughness:{value:1},sheenRoughnessMap:{value:null},sheenRoughnessMapTransform:{value:new ee},transmission:{value:0},transmissionMap:{value:null},transmissionMapTransform:{value:new ee},transmissionSamplerSize:{value:new Wt},transmissionSamplerMap:{value:null},thickness:{value:0},thicknessMap:{value:null},thicknessMapTransform:{value:new ee},attenuationDistance:{value:0},attenuationColor:{value:new _t(0)},specularColor:{value:new _t(1,1,1)},specularColorMap:{value:null},specularColorMapTransform:{value:new ee},specularIntensity:{value:1},specularIntensityMap:{value:null},specularIntensityMapTransform:{value:new ee},anisotropyVector:{value:new Wt},anisotropyMap:{value:null},anisotropyMapTransform:{value:new ee}}]),vertexShader:ie.meshphysical_vert,fragmentShader:ie.meshphysical_frag};var gr={r:0,b:0,g:0},bi=new sn,ym=new ce;function vm(i,t,e,n,s,r,o){let a=new _t(0),c=r===!0?0:1,l,h,u=null,p=0,f=null;function g(x){let _=x.isScene===!0?x.background:null;return _&&_.isTexture&&(_=(x.backgroundBlurriness>0?e:t).get(_)),_}function y(x){let _=!1,v=g(x);v===null?d(a,c):v&&v.isColor&&(d(v,1),_=!0);let C=i.xr.getEnvironmentBlendMode();C==="additive"?n.buffers.color.setClear(0,0,0,1,o):C==="alpha-blend"&&n.buffers.color.setClear(0,0,0,0,o),(i.autoClear||_)&&(n.buffers.depth.setTest(!0),n.buffers.depth.setMask(!0),n.buffers.color.setMask(!0),i.clear(i.autoClearColor,i.autoClearDepth,i.autoClearStencil))}function m(x,_){let v=g(_);v&&(v.isCubeTexture||v.mapping===no)?(h===void 0&&(h=new se(new Qe(1,1,1),new Pe({name:"BackgroundCubeMaterial",uniforms:us(In.backgroundCube.uniforms),vertexShader:In.backgroundCube.vertexShader,fragmentShader:In.backgroundCube.fragmentShader,side:ke,depthTest:!1,depthWrite:!1,fog:!1})),h.geometry.deleteAttribute("normal"),h.geometry.deleteAttribute("uv"),h.onBeforeRender=function(C,w,A){this.matrixWorld.copyPosition(A.matrixWorld)},Object.defineProperty(h.material,"envMap",{get:function(){return this.uniforms.envMap.value}}),s.update(h)),bi.copy(_.backgroundRotation),bi.x*=-1,bi.y*=-1,bi.z*=-1,v.isCubeTexture&&v.isRenderTargetTexture===!1&&(bi.y*=-1,bi.z*=-1),h.material.uniforms.envMap.value=v,h.material.uniforms.flipEnvMap.value=v.isCubeTexture&&v.isRenderTargetTexture===!1?-1:1,h.material.uniforms.backgroundBlurriness.value=_.backgroundBlurriness,h.material.uniforms.backgroundIntensity.value=_.backgroundIntensity,h.material.uniforms.backgroundRotation.value.setFromMatrix4(ym.makeRotationFromEuler(bi)),h.material.toneMapped=he.getTransfer(v.colorSpace)!==xe,(u!==v||p!==v.version||f!==i.toneMapping)&&(h.material.needsUpdate=!0,u=v,p=v.version,f=i.toneMapping),h.layers.enableAll(),x.unshift(h,h.geometry,h.material,0,0,null)):v&&v.isTexture&&(l===void 0&&(l=new se(new wn(2,2),new Pe({name:"BackgroundMaterial",uniforms:us(In.background.uniforms),vertexShader:In.background.vertexShader,fragmentShader:In.background.fragmentShader,side:Dn,depthTest:!1,depthWrite:!1,fog:!1})),l.geometry.deleteAttribute("normal"),Object.defineProperty(l.material,"map",{get:function(){return this.uniforms.t2D.value}}),s.update(l)),l.material.uniforms.t2D.value=v,l.material.uniforms.backgroundIntensity.value=_.backgroundIntensity,l.material.toneMapped=he.getTransfer(v.colorSpace)!==xe,v.matrixAutoUpdate===!0&&v.updateMatrix(),l.material.uniforms.uvTransform.value.copy(v.matrix),(u!==v||p!==v.version||f!==i.toneMapping)&&(l.material.needsUpdate=!0,u=v,p=v.version,f=i.toneMapping),l.layers.enableAll(),x.unshift(l,l.geometry,l.material,0,0,null))}function d(x,_){x.getRGB(gr,dh(i)),n.buffers.color.setClear(gr.r,gr.g,gr.b,_,o)}return{getClearColor:function(){return a},setClearColor:function(x,_=1){a.set(x),c=_,d(a,c)},getClearAlpha:function(){return c},setClearAlpha:function(x){c=x,d(a,c)},render:y,addToRenderList:m}}function Mm(i,t){let e=i.getParameter(i.MAX_VERTEX_ATTRIBS),n={},s=p(null),r=s,o=!1;function a(M,R,F,V,z){let k=!1,I=u(V,F,R);r!==I&&(r=I,l(r.object)),k=f(M,V,F,z),k&&g(M,V,F,z),z!==null&&t.update(z,i.ELEMENT_ARRAY_BUFFER),(k||o)&&(o=!1,v(M,R,F,V),z!==null&&i.bindBuffer(i.ELEMENT_ARRAY_BUFFER,t.get(z).buffer))}function c(){return i.createVertexArray()}function l(M){return i.bindVertexArray(M)}function h(M){return i.deleteVertexArray(M)}function u(M,R,F){let V=F.wireframe===!0,z=n[M.id];z===void 0&&(z={},n[M.id]=z);let k=z[R.id];k===void 0&&(k={},z[R.id]=k);let I=k[V];return I===void 0&&(I=p(c()),k[V]=I),I}function p(M){let R=[],F=[],V=[];for(let z=0;z<e;z++)R[z]=0,F[z]=0,V[z]=0;return{geometry:null,program:null,wireframe:!1,newAttributes:R,enabledAttributes:F,attributeDivisors:V,object:M,attributes:{},index:null}}function f(M,R,F,V){let z=r.attributes,k=R.attributes,I=0,Y=F.getAttributes();for(let B in Y)if(Y[B].location>=0){let ut=z[B],Et=k[B];if(Et===void 0&&(B==="instanceMatrix"&&M.instanceMatrix&&(Et=M.instanceMatrix),B==="instanceColor"&&M.instanceColor&&(Et=M.instanceColor)),ut===void 0||ut.attribute!==Et||Et&&ut.data!==Et.data)return!0;I++}return r.attributesNum!==I||r.index!==V}function g(M,R,F,V){let z={},k=R.attributes,I=0,Y=F.getAttributes();for(let B in Y)if(Y[B].location>=0){let ut=k[B];ut===void 0&&(B==="instanceMatrix"&&M.instanceMatrix&&(ut=M.instanceMatrix),B==="instanceColor"&&M.instanceColor&&(ut=M.instanceColor));let Et={};Et.attribute=ut,ut&&ut.data&&(Et.data=ut.data),z[B]=Et,I++}r.attributes=z,r.attributesNum=I,r.index=V}function y(){let M=r.newAttributes;for(let R=0,F=M.length;R<F;R++)M[R]=0}function m(M){d(M,0)}function d(M,R){let F=r.newAttributes,V=r.enabledAttributes,z=r.attributeDivisors;F[M]=1,V[M]===0&&(i.enableVertexAttribArray(M),V[M]=1),z[M]!==R&&(i.vertexAttribDivisor(M,R),z[M]=R)}function x(){let M=r.newAttributes,R=r.enabledAttributes;for(let F=0,V=R.length;F<V;F++)R[F]!==M[F]&&(i.disableVertexAttribArray(F),R[F]=0)}function _(M,R,F,V,z,k,I){I===!0?i.vertexAttribIPointer(M,R,F,z,k):i.vertexAttribPointer(M,R,F,V,z,k)}function v(M,R,F,V){y();let z=V.attributes,k=F.getAttributes(),I=R.defaultAttributeValues;for(let Y in k){let B=k[Y];if(B.location>=0){let Q=z[Y];if(Q===void 0&&(Y==="instanceMatrix"&&M.instanceMatrix&&(Q=M.instanceMatrix),Y==="instanceColor"&&M.instanceColor&&(Q=M.instanceColor)),Q!==void 0){let ut=Q.normalized,Et=Q.itemSize,zt=t.get(Q);if(zt===void 0)continue;let $t=zt.buffer,H=zt.type,st=zt.bytesPerElement,pt=H===i.INT||H===i.UNSIGNED_INT||Q.gpuType===mc;if(Q.isInterleavedBufferAttribute){let ct=Q.data,Pt=ct.stride,At=Q.offset;if(ct.isInstancedInterleavedBuffer){for(let Ot=0;Ot<B.locationSize;Ot++)d(B.location+Ot,ct.meshPerAttribute);M.isInstancedMesh!==!0&&V._maxInstanceCount===void 0&&(V._maxInstanceCount=ct.meshPerAttribute*ct.count)}else for(let Ot=0;Ot<B.locationSize;Ot++)m(B.location+Ot);i.bindBuffer(i.ARRAY_BUFFER,$t);for(let Ot=0;Ot<B.locationSize;Ot++)_(B.location+Ot,Et/B.locationSize,H,ut,Pt*st,(At+Et/B.locationSize*Ot)*st,pt)}else{if(Q.isInstancedBufferAttribute){for(let ct=0;ct<B.locationSize;ct++)d(B.location+ct,Q.meshPerAttribute);M.isInstancedMesh!==!0&&V._maxInstanceCount===void 0&&(V._maxInstanceCount=Q.meshPerAttribute*Q.count)}else for(let ct=0;ct<B.locationSize;ct++)m(B.location+ct);i.bindBuffer(i.ARRAY_BUFFER,$t);for(let ct=0;ct<B.locationSize;ct++)_(B.location+ct,Et/B.locationSize,H,ut,Et*st,Et/B.locationSize*ct*st,pt)}}else if(I!==void 0){let ut=I[Y];if(ut!==void 0)switch(ut.length){case 2:i.vertexAttrib2fv(B.location,ut);break;case 3:i.vertexAttrib3fv(B.location,ut);break;case 4:i.vertexAttrib4fv(B.location,ut);break;default:i.vertexAttrib1fv(B.location,ut)}}}}x()}function C(){P();for(let M in n){let R=n[M];for(let F in R){let V=R[F];for(let z in V)h(V[z].object),delete V[z];delete R[F]}delete n[M]}}function w(M){if(n[M.id]===void 0)return;let R=n[M.id];for(let F in R){let V=R[F];for(let z in V)h(V[z].object),delete V[z];delete R[F]}delete n[M.id]}function A(M){for(let R in n){let F=n[R];if(F[M.id]===void 0)continue;let V=F[M.id];for(let z in V)h(V[z].object),delete V[z];delete F[M.id]}}function P(){E(),o=!0,r!==s&&(r=s,l(r.object))}function E(){s.geometry=null,s.program=null,s.wireframe=!1}return{setup:a,reset:P,resetDefaultState:E,dispose:C,releaseStatesOfGeometry:w,releaseStatesOfProgram:A,initAttributes:y,enableAttribute:m,disableUnusedAttributes:x}}function Sm(i,t,e){let n;function s(l){n=l}function r(l,h){i.drawArrays(n,l,h),e.update(h,n,1)}function o(l,h,u){u!==0&&(i.drawArraysInstanced(n,l,h,u),e.update(h,n,u))}function a(l,h,u){if(u===0)return;t.get("WEBGL_multi_draw").multiDrawArraysWEBGL(n,l,0,h,0,u);let f=0;for(let g=0;g<u;g++)f+=h[g];e.update(f,n,1)}function c(l,h,u,p){if(u===0)return;let f=t.get("WEBGL_multi_draw");if(f===null)for(let g=0;g<l.length;g++)o(l[g],h[g],p[g]);else{f.multiDrawArraysInstancedWEBGL(n,l,0,h,0,p,0,u);let g=0;for(let y=0;y<u;y++)g+=h[y]*p[y];e.update(g,n,1)}}this.setMode=s,this.render=r,this.renderInstances=o,this.renderMultiDraw=a,this.renderMultiDrawInstances=c}function bm(i,t,e,n){let s;function r(){if(s!==void 0)return s;if(t.has("EXT_texture_filter_anisotropic")===!0){let A=t.get("EXT_texture_filter_anisotropic");s=i.getParameter(A.MAX_TEXTURE_MAX_ANISOTROPY_EXT)}else s=0;return s}function o(A){return!(A!==dn&&n.convert(A)!==i.getParameter(i.IMPLEMENTATION_COLOR_READ_FORMAT))}function a(A){let P=A===Ys&&(t.has("EXT_color_buffer_half_float")||t.has("EXT_color_buffer_float"));return!(A!==Jn&&n.convert(A)!==i.getParameter(i.IMPLEMENTATION_COLOR_READ_TYPE)&&A!==_n&&!P)}function c(A){if(A==="highp"){if(i.getShaderPrecisionFormat(i.VERTEX_SHADER,i.HIGH_FLOAT).precision>0&&i.getShaderPrecisionFormat(i.FRAGMENT_SHADER,i.HIGH_FLOAT).precision>0)return"highp";A="mediump"}return A==="mediump"&&i.getShaderPrecisionFormat(i.VERTEX_SHADER,i.MEDIUM_FLOAT).precision>0&&i.getShaderPrecisionFormat(i.FRAGMENT_SHADER,i.MEDIUM_FLOAT).precision>0?"mediump":"lowp"}let l=e.precision!==void 0?e.precision:"highp",h=c(l);h!==l&&(console.warn("THREE.WebGLRenderer:",l,"not supported, using",h,"instead."),l=h);let u=e.logarithmicDepthBuffer===!0,p=e.reverseDepthBuffer===!0&&t.has("EXT_clip_control"),f=i.getParameter(i.MAX_TEXTURE_IMAGE_UNITS),g=i.getParameter(i.MAX_VERTEX_TEXTURE_IMAGE_UNITS),y=i.getParameter(i.MAX_TEXTURE_SIZE),m=i.getParameter(i.MAX_CUBE_MAP_TEXTURE_SIZE),d=i.getParameter(i.MAX_VERTEX_ATTRIBS),x=i.getParameter(i.MAX_VERTEX_UNIFORM_VECTORS),_=i.getParameter(i.MAX_VARYING_VECTORS),v=i.getParameter(i.MAX_FRAGMENT_UNIFORM_VECTORS),C=g>0,w=i.getParameter(i.MAX_SAMPLES);return{isWebGL2:!0,getMaxAnisotropy:r,getMaxPrecision:c,textureFormatReadable:o,textureTypeReadable:a,precision:l,logarithmicDepthBuffer:u,reverseDepthBuffer:p,maxTextures:f,maxVertexTextures:g,maxTextureSize:y,maxCubemapSize:m,maxAttributes:d,maxVertexUniforms:x,maxVaryings:_,maxFragmentUniforms:v,vertexTextures:C,maxSamples:w}}function Em(i){let t=this,e=null,n=0,s=!1,r=!1,o=new un,a=new ee,c={value:null,needsUpdate:!1};this.uniform=c,this.numPlanes=0,this.numIntersection=0,this.init=function(u,p){let f=u.length!==0||p||n!==0||s;return s=p,n=u.length,f},this.beginShadows=function(){r=!0,h(null)},this.endShadows=function(){r=!1},this.setGlobalState=function(u,p){e=h(u,p,0)},this.setState=function(u,p,f){let g=u.clippingPlanes,y=u.clipIntersection,m=u.clipShadows,d=i.get(u);if(!s||g===null||g.length===0||r&&!m)r?h(null):l();else{let x=r?0:n,_=x*4,v=d.clippingState||null;c.value=v,v=h(g,p,_,f);for(let C=0;C!==_;++C)v[C]=e[C];d.clippingState=v,this.numIntersection=y?this.numPlanes:0,this.numPlanes+=x}};function l(){c.value!==e&&(c.value=e,c.needsUpdate=n>0),t.numPlanes=n,t.numIntersection=0}function h(u,p,f,g){let y=u!==null?u.length:0,m=null;if(y!==0){if(m=c.value,g!==!0||m===null){let d=f+y*4,x=p.matrixWorldInverse;a.getNormalMatrix(x),(m===null||m.length<d)&&(m=new Float32Array(d));for(let _=0,v=f;_!==y;++_,v+=4)o.copy(u[_]).applyMatrix4(x,a),o.normal.toArray(m,v),m[v+3]=o.constant}c.value=m,c.needsUpdate=!0}return t.numPlanes=y,t.numIntersection=0,m}}function Tm(i){let t=new WeakMap;function e(o,a){return a===ra?o.mapping=os:a===oa&&(o.mapping=as),o}function n(o){if(o&&o.isTexture){let a=o.mapping;if(a===ra||a===oa)if(t.has(o)){let c=t.get(o).texture;return e(c,o.mapping)}else{let c=o.image;if(c&&c.height>0){let l=new ka(c.height);return l.fromEquirectangularTexture(i,o),t.set(o,l),o.addEventListener("dispose",s),e(l.texture,o.mapping)}else return null}}return o}function s(o){let a=o.target;a.removeEventListener("dispose",s);let c=t.get(a);c!==void 0&&(t.delete(a),c.dispose())}function r(){t=new WeakMap}return{get:n,dispose:r}}var Hr=class extends zr{constructor(t=-1,e=1,n=1,s=-1,r=.1,o=2e3){super(),this.isOrthographicCamera=!0,this.type="OrthographicCamera",this.zoom=1,this.view=null,this.left=t,this.right=e,this.top=n,this.bottom=s,this.near=r,this.far=o,this.updateProjectionMatrix()}copy(t,e){return super.copy(t,e),this.left=t.left,this.right=t.right,this.top=t.top,this.bottom=t.bottom,this.near=t.near,this.far=t.far,this.zoom=t.zoom,this.view=t.view===null?null:Object.assign({},t.view),this}setViewOffset(t,e,n,s,r,o){this.view===null&&(this.view={enabled:!0,fullWidth:1,fullHeight:1,offsetX:0,offsetY:0,width:1,height:1}),this.view.enabled=!0,this.view.fullWidth=t,this.view.fullHeight=e,this.view.offsetX=n,this.view.offsetY=s,this.view.width=r,this.view.height=o,this.updateProjectionMatrix()}clearViewOffset(){this.view!==null&&(this.view.enabled=!1),this.updateProjectionMatrix()}updateProjectionMatrix(){let t=(this.right-this.left)/(2*this.zoom),e=(this.top-this.bottom)/(2*this.zoom),n=(this.right+this.left)/2,s=(this.top+this.bottom)/2,r=n-t,o=n+t,a=s+e,c=s-e;if(this.view!==null&&this.view.enabled){let l=(this.right-this.left)/this.view.fullWidth/this.zoom,h=(this.top-this.bottom)/this.view.fullHeight/this.zoom;r+=l*this.view.offsetX,o=r+l*this.view.width,a-=h*this.view.offsetY,c=a-h*this.view.height}this.projectionMatrix.makeOrthographic(r,o,a,c,this.near,this.far,this.coordinateSystem),this.projectionMatrixInverse.copy(this.projectionMatrix).invert()}toJSON(t){let e=super.toJSON(t);return e.object.zoom=this.zoom,e.object.left=this.left,e.object.right=this.right,e.object.top=this.top,e.object.bottom=this.bottom,e.object.near=this.near,e.object.far=this.far,this.view!==null&&(e.object.view=Object.assign({},this.view)),e}},ts=4,yl=[.125,.215,.35,.446,.526,.582],Ai=20,Vo=new Hr,vl=new _t,Go=null,Wo=0,Xo=0,Yo=!1,Ti=(1+Math.sqrt(5))/2,$i=1/Ti,Ml=[new U(-Ti,$i,0),new U(Ti,$i,0),new U(-$i,0,Ti),new U($i,0,Ti),new U(0,Ti,-$i),new U(0,Ti,$i),new U(-1,1,-1),new U(1,1,-1),new U(-1,1,1),new U(1,1,1)],ds=class{constructor(t){this._renderer=t,this._pingPongRenderTarget=null,this._lodMax=0,this._cubeSize=0,this._lodPlanes=[],this._sizeLods=[],this._sigmas=[],this._blurMaterial=null,this._cubemapMaterial=null,this._equirectMaterial=null,this._compileMaterial(this._blurMaterial)}fromScene(t,e=0,n=.1,s=100){Go=this._renderer.getRenderTarget(),Wo=this._renderer.getActiveCubeFace(),Xo=this._renderer.getActiveMipmapLevel(),Yo=this._renderer.xr.enabled,this._renderer.xr.enabled=!1,this._setSize(256);let r=this._allocateTargets();return r.depthBuffer=!0,this._sceneToCubeUV(t,n,s,r),e>0&&this._blur(r,0,0,e),this._applyPMREM(r),this._cleanup(r),r}fromEquirectangular(t,e=null){return this._fromTexture(t,e)}fromCubemap(t,e=null){return this._fromTexture(t,e)}compileCubemapShader(){this._cubemapMaterial===null&&(this._cubemapMaterial=El(),this._compileMaterial(this._cubemapMaterial))}compileEquirectangularShader(){this._equirectMaterial===null&&(this._equirectMaterial=bl(),this._compileMaterial(this._equirectMaterial))}dispose(){this._dispose(),this._cubemapMaterial!==null&&this._cubemapMaterial.dispose(),this._equirectMaterial!==null&&this._equirectMaterial.dispose()}_setSize(t){this._lodMax=Math.floor(Math.log2(t)),this._cubeSize=Math.pow(2,this._lodMax)}_dispose(){this._blurMaterial!==null&&this._blurMaterial.dispose(),this._pingPongRenderTarget!==null&&this._pingPongRenderTarget.dispose();for(let t=0;t<this._lodPlanes.length;t++)this._lodPlanes[t].dispose()}_cleanup(t){this._renderer.setRenderTarget(Go,Wo,Xo),this._renderer.xr.enabled=Yo,t.scissorTest=!1,_r(t,0,0,t.width,t.height)}_fromTexture(t,e){t.mapping===os||t.mapping===as?this._setSize(t.image.length===0?16:t.image[0].width||t.image[0].image.width):this._setSize(t.image.width/4),Go=this._renderer.getRenderTarget(),Wo=this._renderer.getActiveCubeFace(),Xo=this._renderer.getActiveMipmapLevel(),Yo=this._renderer.xr.enabled,this._renderer.xr.enabled=!1;let n=e||this._allocateTargets();return this._textureToCubeUV(t,n),this._applyPMREM(n),this._cleanup(n),n}_allocateTargets(){let t=3*Math.max(this._cubeSize,112),e=4*this._cubeSize,n={magFilter:je,minFilter:je,generateMipmaps:!1,type:Ys,format:dn,colorSpace:_s,depthBuffer:!1},s=Sl(t,e,n);if(this._pingPongRenderTarget===null||this._pingPongRenderTarget.width!==t||this._pingPongRenderTarget.height!==e){this._pingPongRenderTarget!==null&&this._dispose(),this._pingPongRenderTarget=Sl(t,e,n);let{_lodMax:r}=this;({sizeLods:this._sizeLods,lodPlanes:this._lodPlanes,sigmas:this._sigmas}=wm(r)),this._blurMaterial=Am(r,t,e)}return s}_compileMaterial(t){let e=new se(this._lodPlanes[0],t);this._renderer.compile(e,Vo)}_sceneToCubeUV(t,e,n,s){let a=new Ve(90,1,e,n),c=[1,-1,1,1,1,1],l=[1,1,1,-1,-1,-1],h=this._renderer,u=h.autoClear,p=h.toneMapping;h.getClearColor(vl),h.toneMapping=fi,h.autoClear=!1;let f=new Tn({name:"PMREM.Background",side:ke,depthWrite:!1,depthTest:!1}),g=new se(new Qe,f),y=!1,m=t.background;m?m.isColor&&(f.color.copy(m),t.background=null,y=!0):(f.color.copy(vl),y=!0);for(let d=0;d<6;d++){let x=d%3;x===0?(a.up.set(0,c[d],0),a.lookAt(l[d],0,0)):x===1?(a.up.set(0,0,c[d]),a.lookAt(0,l[d],0)):(a.up.set(0,c[d],0),a.lookAt(0,0,l[d]));let _=this._cubeSize;_r(s,x*_,d>2?_:0,_,_),h.setRenderTarget(s),y&&h.render(g,a),h.render(t,a)}g.geometry.dispose(),g.material.dispose(),h.toneMapping=p,h.autoClear=u,t.background=m}_textureToCubeUV(t,e){let n=this._renderer,s=t.mapping===os||t.mapping===as;s?(this._cubemapMaterial===null&&(this._cubemapMaterial=El()),this._cubemapMaterial.uniforms.flipEnvMap.value=t.isRenderTargetTexture===!1?-1:1):this._equirectMaterial===null&&(this._equirectMaterial=bl());let r=s?this._cubemapMaterial:this._equirectMaterial,o=new se(this._lodPlanes[0],r),a=r.uniforms;a.envMap.value=t;let c=this._cubeSize;_r(e,0,0,3*c,2*c),n.setRenderTarget(e),n.render(o,Vo)}_applyPMREM(t){let e=this._renderer,n=e.autoClear;e.autoClear=!1;let s=this._lodPlanes.length;for(let r=1;r<s;r++){let o=Math.sqrt(this._sigmas[r]*this._sigmas[r]-this._sigmas[r-1]*this._sigmas[r-1]),a=Ml[(s-r-1)%Ml.length];this._blur(t,r-1,r,o,a)}e.autoClear=n}_blur(t,e,n,s,r){let o=this._pingPongRenderTarget;this._halfBlur(t,o,e,n,s,"latitudinal",r),this._halfBlur(o,t,n,n,s,"longitudinal",r)}_halfBlur(t,e,n,s,r,o,a){let c=this._renderer,l=this._blurMaterial;o!=="latitudinal"&&o!=="longitudinal"&&console.error("blur direction must be either latitudinal or longitudinal!");let h=3,u=new se(this._lodPlanes[s],l),p=l.uniforms,f=this._sizeLods[n]-1,g=isFinite(r)?Math.PI/(2*f):2*Math.PI/(2*Ai-1),y=r/g,m=isFinite(r)?1+Math.floor(h*y):Ai;m>Ai&&console.warn(`sigmaRadians, ${r}, is too large and will clip, as it requested ${m} samples when the maximum is set to ${Ai}`);let d=[],x=0;for(let A=0;A<Ai;++A){let P=A/y,E=Math.exp(-P*P/2);d.push(E),A===0?x+=E:A<m&&(x+=2*E)}for(let A=0;A<d.length;A++)d[A]=d[A]/x;p.envMap.value=t.texture,p.samples.value=m,p.weights.value=d,p.latitudinal.value=o==="latitudinal",a&&(p.poleAxis.value=a);let{_lodMax:_}=this;p.dTheta.value=g,p.mipInt.value=_-n;let v=this._sizeLods[s],C=3*v*(s>_-ts?s-_+ts:0),w=4*(this._cubeSize-v);_r(e,C,w,3*v,2*v),c.setRenderTarget(e),c.render(u,Vo)}};function wm(i){let t=[],e=[],n=[],s=i,r=i-ts+1+yl.length;for(let o=0;o<r;o++){let a=Math.pow(2,s);e.push(a);let c=1/a;o>i-ts?c=yl[o-i+ts-1]:o===0&&(c=0),n.push(c);let l=1/(a-2),h=-l,u=1+l,p=[h,h,u,h,u,u,h,h,u,u,h,u],f=6,g=6,y=3,m=2,d=1,x=new Float32Array(y*g*f),_=new Float32Array(m*g*f),v=new Float32Array(d*g*f);for(let w=0;w<f;w++){let A=w%3*2/3-1,P=w>2?0:-1,E=[A,P,0,A+2/3,P,0,A+2/3,P+1,0,A,P,0,A+2/3,P+1,0,A,P+1,0];x.set(E,y*g*w),_.set(p,m*g*w);let M=[w,w,w,w,w,w];v.set(M,d*g*w)}let C=new Ge;C.setAttribute("position",new ve(x,y)),C.setAttribute("uv",new ve(_,m)),C.setAttribute("faceIndex",new ve(v,d)),t.push(C),s>ts&&s--}return{lodPlanes:t,sizeLods:e,sigmas:n}}function Sl(i,t,e){let n=new En(i,t,e);return n.texture.mapping=no,n.texture.name="PMREM.cubeUv",n.scissorTest=!0,n}function _r(i,t,e,n,s){i.viewport.set(t,e,n,s),i.scissor.set(t,e,n,s)}function Am(i,t,e){let n=new Float32Array(Ai),s=new U(0,1,0);return new Pe({name:"SphericalGaussianBlur",defines:{n:Ai,CUBEUV_TEXEL_WIDTH:1/t,CUBEUV_TEXEL_HEIGHT:1/e,CUBEUV_MAX_MIP:`${i}.0`},uniforms:{envMap:{value:null},samples:{value:1},weights:{value:n},latitudinal:{value:!1},dTheta:{value:0},mipInt:{value:0},poleAxis:{value:s}},vertexShader:bc(),fragmentShader:`

			precision mediump float;
			precision mediump int;

			varying vec3 vOutputDirection;

			uniform sampler2D envMap;
			uniform int samples;
			uniform float weights[ n ];
			uniform bool latitudinal;
			uniform float dTheta;
			uniform float mipInt;
			uniform vec3 poleAxis;

			#define ENVMAP_TYPE_CUBE_UV
			#include <cube_uv_reflection_fragment>

			vec3 getSample( float theta, vec3 axis ) {

				float cosTheta = cos( theta );
				// Rodrigues' axis-angle rotation
				vec3 sampleDirection = vOutputDirection * cosTheta
					+ cross( axis, vOutputDirection ) * sin( theta )
					+ axis * dot( axis, vOutputDirection ) * ( 1.0 - cosTheta );

				return bilinearCubeUV( envMap, sampleDirection, mipInt );

			}

			void main() {

				vec3 axis = latitudinal ? poleAxis : cross( poleAxis, vOutputDirection );

				if ( all( equal( axis, vec3( 0.0 ) ) ) ) {

					axis = vec3( vOutputDirection.z, 0.0, - vOutputDirection.x );

				}

				axis = normalize( axis );

				gl_FragColor = vec4( 0.0, 0.0, 0.0, 1.0 );
				gl_FragColor.rgb += weights[ 0 ] * getSample( 0.0, axis );

				for ( int i = 1; i < n; i++ ) {

					if ( i >= samples ) {

						break;

					}

					float theta = dTheta * float( i );
					gl_FragColor.rgb += weights[ i ] * getSample( -1.0 * theta, axis );
					gl_FragColor.rgb += weights[ i ] * getSample( theta, axis );

				}

			}
		`,blending:di,depthTest:!1,depthWrite:!1})}function bl(){return new Pe({name:"EquirectangularToCubeUV",uniforms:{envMap:{value:null}},vertexShader:bc(),fragmentShader:`

			precision mediump float;
			precision mediump int;

			varying vec3 vOutputDirection;

			uniform sampler2D envMap;

			#include <common>

			void main() {

				vec3 outputDirection = normalize( vOutputDirection );
				vec2 uv = equirectUv( outputDirection );

				gl_FragColor = vec4( texture2D ( envMap, uv ).rgb, 1.0 );

			}
		`,blending:di,depthTest:!1,depthWrite:!1})}function El(){return new Pe({name:"CubemapToCubeUV",uniforms:{envMap:{value:null},flipEnvMap:{value:-1}},vertexShader:bc(),fragmentShader:`

			precision mediump float;
			precision mediump int;

			uniform float flipEnvMap;

			varying vec3 vOutputDirection;

			uniform samplerCube envMap;

			void main() {

				gl_FragColor = textureCube( envMap, vec3( flipEnvMap * vOutputDirection.x, vOutputDirection.yz ) );

			}
		`,blending:di,depthTest:!1,depthWrite:!1})}function bc(){return`

		precision mediump float;
		precision mediump int;

		attribute float faceIndex;

		varying vec3 vOutputDirection;

		// RH coordinate system; PMREM face-indexing convention
		vec3 getDirection( vec2 uv, float face ) {

			uv = 2.0 * uv - 1.0;

			vec3 direction = vec3( uv, 1.0 );

			if ( face == 0.0 ) {

				direction = direction.zyx; // ( 1, v, u ) pos x

			} else if ( face == 1.0 ) {

				direction = direction.xzy;
				direction.xz *= -1.0; // ( -u, 1, -v ) pos y

			} else if ( face == 2.0 ) {

				direction.x *= -1.0; // ( -u, v, 1 ) pos z

			} else if ( face == 3.0 ) {

				direction = direction.zyx;
				direction.xz *= -1.0; // ( -1, v, -u ) neg x

			} else if ( face == 4.0 ) {

				direction = direction.xzy;
				direction.xy *= -1.0; // ( -u, -1, v ) neg y

			} else if ( face == 5.0 ) {

				direction.z *= -1.0; // ( u, v, -1 ) neg z

			}

			return direction;

		}

		void main() {

			vOutputDirection = getDirection( uv, faceIndex );
			gl_Position = vec4( position, 1.0 );

		}
	`}function Cm(i){let t=new WeakMap,e=null;function n(a){if(a&&a.isTexture){let c=a.mapping,l=c===ra||c===oa,h=c===os||c===as;if(l||h){let u=t.get(a),p=u!==void 0?u.texture.pmremVersion:0;if(a.isRenderTargetTexture&&a.pmremVersion!==p)return e===null&&(e=new ds(i)),u=l?e.fromEquirectangular(a,u):e.fromCubemap(a,u),u.texture.pmremVersion=a.pmremVersion,t.set(a,u),u.texture;if(u!==void 0)return u.texture;{let f=a.image;return l&&f&&f.height>0||h&&f&&s(f)?(e===null&&(e=new ds(i)),u=l?e.fromEquirectangular(a):e.fromCubemap(a),u.texture.pmremVersion=a.pmremVersion,t.set(a,u),a.addEventListener("dispose",r),u.texture):null}}}return a}function s(a){let c=0,l=6;for(let h=0;h<l;h++)a[h]!==void 0&&c++;return c===l}function r(a){let c=a.target;c.removeEventListener("dispose",r);let l=t.get(c);l!==void 0&&(t.delete(c),l.dispose())}function o(){t=new WeakMap,e!==null&&(e.dispose(),e=null)}return{get:n,dispose:o}}function Rm(i){let t={};function e(n){if(t[n]!==void 0)return t[n];let s;switch(n){case"WEBGL_depth_texture":s=i.getExtension("WEBGL_depth_texture")||i.getExtension("MOZ_WEBGL_depth_texture")||i.getExtension("WEBKIT_WEBGL_depth_texture");break;case"EXT_texture_filter_anisotropic":s=i.getExtension("EXT_texture_filter_anisotropic")||i.getExtension("MOZ_EXT_texture_filter_anisotropic")||i.getExtension("WEBKIT_EXT_texture_filter_anisotropic");break;case"WEBGL_compressed_texture_s3tc":s=i.getExtension("WEBGL_compressed_texture_s3tc")||i.getExtension("MOZ_WEBGL_compressed_texture_s3tc")||i.getExtension("WEBKIT_WEBGL_compressed_texture_s3tc");break;case"WEBGL_compressed_texture_pvrtc":s=i.getExtension("WEBGL_compressed_texture_pvrtc")||i.getExtension("WEBKIT_WEBGL_compressed_texture_pvrtc");break;default:s=i.getExtension(n)}return t[n]=s,s}return{has:function(n){return e(n)!==null},init:function(){e("EXT_color_buffer_float"),e("WEBGL_clip_cull_distance"),e("OES_texture_float_linear"),e("EXT_color_buffer_half_float"),e("WEBGL_multisampled_render_to_texture"),e("WEBGL_render_shared_exponent")},get:function(n){let s=e(n);return s===null&&Ds("THREE.WebGLRenderer: "+n+" extension not supported."),s}}}function Pm(i,t,e,n){let s={},r=new WeakMap;function o(u){let p=u.target;p.index!==null&&t.remove(p.index);for(let g in p.attributes)t.remove(p.attributes[g]);for(let g in p.morphAttributes){let y=p.morphAttributes[g];for(let m=0,d=y.length;m<d;m++)t.remove(y[m])}p.removeEventListener("dispose",o),delete s[p.id];let f=r.get(p);f&&(t.remove(f),r.delete(p)),n.releaseStatesOfGeometry(p),p.isInstancedBufferGeometry===!0&&delete p._maxInstanceCount,e.memory.geometries--}function a(u,p){return s[p.id]===!0||(p.addEventListener("dispose",o),s[p.id]=!0,e.memory.geometries++),p}function c(u){let p=u.attributes;for(let g in p)t.update(p[g],i.ARRAY_BUFFER);let f=u.morphAttributes;for(let g in f){let y=f[g];for(let m=0,d=y.length;m<d;m++)t.update(y[m],i.ARRAY_BUFFER)}}function l(u){let p=[],f=u.index,g=u.attributes.position,y=0;if(f!==null){let x=f.array;y=f.version;for(let _=0,v=x.length;_<v;_+=3){let C=x[_+0],w=x[_+1],A=x[_+2];p.push(C,w,w,A,A,C)}}else if(g!==void 0){let x=g.array;y=g.version;for(let _=0,v=x.length/3-1;_<v;_+=3){let C=_+0,w=_+1,A=_+2;p.push(C,w,w,A,A,C)}}else return;let m=new(hh(p)?Br:Or)(p,1);m.version=y;let d=r.get(u);d&&t.remove(d),r.set(u,m)}function h(u){let p=r.get(u);if(p){let f=u.index;f!==null&&p.version<f.version&&l(u)}else l(u);return r.get(u)}return{get:a,update:c,getWireframeAttribute:h}}function Im(i,t,e){let n;function s(p){n=p}let r,o;function a(p){r=p.type,o=p.bytesPerElement}function c(p,f){i.drawElements(n,f,r,p*o),e.update(f,n,1)}function l(p,f,g){g!==0&&(i.drawElementsInstanced(n,f,r,p*o,g),e.update(f,n,g))}function h(p,f,g){if(g===0)return;t.get("WEBGL_multi_draw").multiDrawElementsWEBGL(n,f,0,r,p,0,g);let m=0;for(let d=0;d<g;d++)m+=f[d];e.update(m,n,1)}function u(p,f,g,y){if(g===0)return;let m=t.get("WEBGL_multi_draw");if(m===null)for(let d=0;d<p.length;d++)l(p[d]/o,f[d],y[d]);else{m.multiDrawElementsInstancedWEBGL(n,f,0,r,p,0,y,0,g);let d=0;for(let x=0;x<g;x++)d+=f[x]*y[x];e.update(d,n,1)}}this.setMode=s,this.setIndex=a,this.render=c,this.renderInstances=l,this.renderMultiDraw=h,this.renderMultiDrawInstances=u}function Lm(i){let t={geometries:0,textures:0},e={frame:0,calls:0,triangles:0,points:0,lines:0};function n(r,o,a){switch(e.calls++,o){case i.TRIANGLES:e.triangles+=a*(r/3);break;case i.LINES:e.lines+=a*(r/2);break;case i.LINE_STRIP:e.lines+=a*(r-1);break;case i.LINE_LOOP:e.lines+=a*r;break;case i.POINTS:e.points+=a*r;break;default:console.error("THREE.WebGLInfo: Unknown draw mode:",o);break}}function s(){e.calls=0,e.triangles=0,e.points=0,e.lines=0}return{memory:t,render:e,programs:null,autoReset:!0,reset:s,update:n}}function Dm(i,t,e){let n=new WeakMap,s=new be;function r(o,a,c){let l=o.morphTargetInfluences,h=a.morphAttributes.position||a.morphAttributes.normal||a.morphAttributes.color,u=h!==void 0?h.length:0,p=n.get(a);if(p===void 0||p.count!==u){let E=function(){A.dispose(),n.delete(a),a.removeEventListener("dispose",E)};p!==void 0&&p.texture.dispose();let f=a.morphAttributes.position!==void 0,g=a.morphAttributes.normal!==void 0,y=a.morphAttributes.color!==void 0,m=a.morphAttributes.position||[],d=a.morphAttributes.normal||[],x=a.morphAttributes.color||[],_=0;f===!0&&(_=1),g===!0&&(_=2),y===!0&&(_=3);let v=a.attributes.position.count*_,C=1;v>t.maxTextureSize&&(C=Math.ceil(v/t.maxTextureSize),v=t.maxTextureSize);let w=new Float32Array(v*C*4*u),A=new Nr(w,v,C,u);A.type=_n,A.needsUpdate=!0;let P=_*4;for(let M=0;M<u;M++){let R=m[M],F=d[M],V=x[M],z=v*C*4*M;for(let k=0;k<R.count;k++){let I=k*P;f===!0&&(s.fromBufferAttribute(R,k),w[z+I+0]=s.x,w[z+I+1]=s.y,w[z+I+2]=s.z,w[z+I+3]=0),g===!0&&(s.fromBufferAttribute(F,k),w[z+I+4]=s.x,w[z+I+5]=s.y,w[z+I+6]=s.z,w[z+I+7]=0),y===!0&&(s.fromBufferAttribute(V,k),w[z+I+8]=s.x,w[z+I+9]=s.y,w[z+I+10]=s.z,w[z+I+11]=V.itemSize===4?s.w:1)}}p={count:u,texture:A,size:new Wt(v,C)},n.set(a,p),a.addEventListener("dispose",E)}if(o.isInstancedMesh===!0&&o.morphTexture!==null)c.getUniforms().setValue(i,"morphTexture",o.morphTexture,e);else{let f=0;for(let y=0;y<l.length;y++)f+=l[y];let g=a.morphTargetsRelative?1:1-f;c.getUniforms().setValue(i,"morphTargetBaseInfluence",g),c.getUniforms().setValue(i,"morphTargetInfluences",l)}c.getUniforms().setValue(i,"morphTargetsTexture",p.texture,e),c.getUniforms().setValue(i,"morphTargetsTextureSize",p.size)}return{update:r}}function Um(i,t,e,n){let s=new WeakMap;function r(c){let l=n.render.frame,h=c.geometry,u=t.get(c,h);if(s.get(u)!==l&&(t.update(u),s.set(u,l)),c.isInstancedMesh&&(c.hasEventListener("dispose",a)===!1&&c.addEventListener("dispose",a),s.get(c)!==l&&(e.update(c.instanceMatrix,i.ARRAY_BUFFER),c.instanceColor!==null&&e.update(c.instanceColor,i.ARRAY_BUFFER),s.set(c,l))),c.isSkinnedMesh){let p=c.skeleton;s.get(p)!==l&&(p.update(),s.set(p,l))}return u}function o(){s=new WeakMap}function a(c){let l=c.target;l.removeEventListener("dispose",a),e.remove(l.instanceMatrix),l.instanceColor!==null&&e.remove(l.instanceColor)}return{update:r,dispose:o}}var Vr=class extends nn{constructor(t,e,n,s,r,o,a,c,l,h=ns){if(h!==ns&&h!==ls)throw new Error("DepthTexture format must be either THREE.DepthFormat or THREE.DepthStencilFormat");n===void 0&&h===ns&&(n=Ci),n===void 0&&h===ls&&(n=cs),super(null,s,r,o,a,c,h,n,l),this.isDepthTexture=!0,this.image={width:t,height:e},this.magFilter=a!==void 0?a:fn,this.minFilter=c!==void 0?c:fn,this.flipY=!1,this.generateMipmaps=!1,this.compareFunction=null}copy(t){return super.copy(t),this.compareFunction=t.compareFunction,this}toJSON(t){let e=super.toJSON(t);return this.compareFunction!==null&&(e.compareFunction=this.compareFunction),e}},ph=new nn,Tl=new Vr(1,1),mh=new Nr,gh=new Ba,_h=new kr,wl=[],Al=[],Cl=new Float32Array(16),Rl=new Float32Array(9),Pl=new Float32Array(4);function ys(i,t,e){let n=i[0];if(n<=0||n>0)return i;let s=t*e,r=wl[s];if(r===void 0&&(r=new Float32Array(s),wl[s]=r),t!==0){n.toArray(r,0);for(let o=1,a=0;o!==t;++o)a+=e,i[o].toArray(r,a)}return r}function Ie(i,t){if(i.length!==t.length)return!1;for(let e=0,n=i.length;e<n;e++)if(i[e]!==t[e])return!1;return!0}function Le(i,t){for(let e=0,n=t.length;e<n;e++)i[e]=t[e]}function ro(i,t){let e=Al[t];e===void 0&&(e=new Int32Array(t),Al[t]=e);for(let n=0;n!==t;++n)e[n]=i.allocateTextureUnit();return e}function Nm(i,t){let e=this.cache;e[0]!==t&&(i.uniform1f(this.addr,t),e[0]=t)}function Fm(i,t){let e=this.cache;if(t.x!==void 0)(e[0]!==t.x||e[1]!==t.y)&&(i.uniform2f(this.addr,t.x,t.y),e[0]=t.x,e[1]=t.y);else{if(Ie(e,t))return;i.uniform2fv(this.addr,t),Le(e,t)}}function Om(i,t){let e=this.cache;if(t.x!==void 0)(e[0]!==t.x||e[1]!==t.y||e[2]!==t.z)&&(i.uniform3f(this.addr,t.x,t.y,t.z),e[0]=t.x,e[1]=t.y,e[2]=t.z);else if(t.r!==void 0)(e[0]!==t.r||e[1]!==t.g||e[2]!==t.b)&&(i.uniform3f(this.addr,t.r,t.g,t.b),e[0]=t.r,e[1]=t.g,e[2]=t.b);else{if(Ie(e,t))return;i.uniform3fv(this.addr,t),Le(e,t)}}function Bm(i,t){let e=this.cache;if(t.x!==void 0)(e[0]!==t.x||e[1]!==t.y||e[2]!==t.z||e[3]!==t.w)&&(i.uniform4f(this.addr,t.x,t.y,t.z,t.w),e[0]=t.x,e[1]=t.y,e[2]=t.z,e[3]=t.w);else{if(Ie(e,t))return;i.uniform4fv(this.addr,t),Le(e,t)}}function zm(i,t){let e=this.cache,n=t.elements;if(n===void 0){if(Ie(e,t))return;i.uniformMatrix2fv(this.addr,!1,t),Le(e,t)}else{if(Ie(e,n))return;Pl.set(n),i.uniformMatrix2fv(this.addr,!1,Pl),Le(e,n)}}function km(i,t){let e=this.cache,n=t.elements;if(n===void 0){if(Ie(e,t))return;i.uniformMatrix3fv(this.addr,!1,t),Le(e,t)}else{if(Ie(e,n))return;Rl.set(n),i.uniformMatrix3fv(this.addr,!1,Rl),Le(e,n)}}function Hm(i,t){let e=this.cache,n=t.elements;if(n===void 0){if(Ie(e,t))return;i.uniformMatrix4fv(this.addr,!1,t),Le(e,t)}else{if(Ie(e,n))return;Cl.set(n),i.uniformMatrix4fv(this.addr,!1,Cl),Le(e,n)}}function Vm(i,t){let e=this.cache;e[0]!==t&&(i.uniform1i(this.addr,t),e[0]=t)}function Gm(i,t){let e=this.cache;if(t.x!==void 0)(e[0]!==t.x||e[1]!==t.y)&&(i.uniform2i(this.addr,t.x,t.y),e[0]=t.x,e[1]=t.y);else{if(Ie(e,t))return;i.uniform2iv(this.addr,t),Le(e,t)}}function Wm(i,t){let e=this.cache;if(t.x!==void 0)(e[0]!==t.x||e[1]!==t.y||e[2]!==t.z)&&(i.uniform3i(this.addr,t.x,t.y,t.z),e[0]=t.x,e[1]=t.y,e[2]=t.z);else{if(Ie(e,t))return;i.uniform3iv(this.addr,t),Le(e,t)}}function Xm(i,t){let e=this.cache;if(t.x!==void 0)(e[0]!==t.x||e[1]!==t.y||e[2]!==t.z||e[3]!==t.w)&&(i.uniform4i(this.addr,t.x,t.y,t.z,t.w),e[0]=t.x,e[1]=t.y,e[2]=t.z,e[3]=t.w);else{if(Ie(e,t))return;i.uniform4iv(this.addr,t),Le(e,t)}}function Ym(i,t){let e=this.cache;e[0]!==t&&(i.uniform1ui(this.addr,t),e[0]=t)}function qm(i,t){let e=this.cache;if(t.x!==void 0)(e[0]!==t.x||e[1]!==t.y)&&(i.uniform2ui(this.addr,t.x,t.y),e[0]=t.x,e[1]=t.y);else{if(Ie(e,t))return;i.uniform2uiv(this.addr,t),Le(e,t)}}function $m(i,t){let e=this.cache;if(t.x!==void 0)(e[0]!==t.x||e[1]!==t.y||e[2]!==t.z)&&(i.uniform3ui(this.addr,t.x,t.y,t.z),e[0]=t.x,e[1]=t.y,e[2]=t.z);else{if(Ie(e,t))return;i.uniform3uiv(this.addr,t),Le(e,t)}}function Zm(i,t){let e=this.cache;if(t.x!==void 0)(e[0]!==t.x||e[1]!==t.y||e[2]!==t.z||e[3]!==t.w)&&(i.uniform4ui(this.addr,t.x,t.y,t.z,t.w),e[0]=t.x,e[1]=t.y,e[2]=t.z,e[3]=t.w);else{if(Ie(e,t))return;i.uniform4uiv(this.addr,t),Le(e,t)}}function Km(i,t,e){let n=this.cache,s=e.allocateTextureUnit();n[0]!==s&&(i.uniform1i(this.addr,s),n[0]=s);let r;this.type===i.SAMPLER_2D_SHADOW?(Tl.compareFunction=lh,r=Tl):r=ph,e.setTexture2D(t||r,s)}function Jm(i,t,e){let n=this.cache,s=e.allocateTextureUnit();n[0]!==s&&(i.uniform1i(this.addr,s),n[0]=s),e.setTexture3D(t||gh,s)}function jm(i,t,e){let n=this.cache,s=e.allocateTextureUnit();n[0]!==s&&(i.uniform1i(this.addr,s),n[0]=s),e.setTextureCube(t||_h,s)}function Qm(i,t,e){let n=this.cache,s=e.allocateTextureUnit();n[0]!==s&&(i.uniform1i(this.addr,s),n[0]=s),e.setTexture2DArray(t||mh,s)}function tg(i){switch(i){case 5126:return Nm;case 35664:return Fm;case 35665:return Om;case 35666:return Bm;case 35674:return zm;case 35675:return km;case 35676:return Hm;case 5124:case 35670:return Vm;case 35667:case 35671:return Gm;case 35668:case 35672:return Wm;case 35669:case 35673:return Xm;case 5125:return Ym;case 36294:return qm;case 36295:return $m;case 36296:return Zm;case 35678:case 36198:case 36298:case 36306:case 35682:return Km;case 35679:case 36299:case 36307:return Jm;case 35680:case 36300:case 36308:case 36293:return jm;case 36289:case 36303:case 36311:case 36292:return Qm}}function eg(i,t){i.uniform1fv(this.addr,t)}function ng(i,t){let e=ys(t,this.size,2);i.uniform2fv(this.addr,e)}function ig(i,t){let e=ys(t,this.size,3);i.uniform3fv(this.addr,e)}function sg(i,t){let e=ys(t,this.size,4);i.uniform4fv(this.addr,e)}function rg(i,t){let e=ys(t,this.size,4);i.uniformMatrix2fv(this.addr,!1,e)}function og(i,t){let e=ys(t,this.size,9);i.uniformMatrix3fv(this.addr,!1,e)}function ag(i,t){let e=ys(t,this.size,16);i.uniformMatrix4fv(this.addr,!1,e)}function cg(i,t){i.uniform1iv(this.addr,t)}function lg(i,t){i.uniform2iv(this.addr,t)}function hg(i,t){i.uniform3iv(this.addr,t)}function ug(i,t){i.uniform4iv(this.addr,t)}function dg(i,t){i.uniform1uiv(this.addr,t)}function fg(i,t){i.uniform2uiv(this.addr,t)}function pg(i,t){i.uniform3uiv(this.addr,t)}function mg(i,t){i.uniform4uiv(this.addr,t)}function gg(i,t,e){let n=this.cache,s=t.length,r=ro(e,s);Ie(n,r)||(i.uniform1iv(this.addr,r),Le(n,r));for(let o=0;o!==s;++o)e.setTexture2D(t[o]||ph,r[o])}function _g(i,t,e){let n=this.cache,s=t.length,r=ro(e,s);Ie(n,r)||(i.uniform1iv(this.addr,r),Le(n,r));for(let o=0;o!==s;++o)e.setTexture3D(t[o]||gh,r[o])}function xg(i,t,e){let n=this.cache,s=t.length,r=ro(e,s);Ie(n,r)||(i.uniform1iv(this.addr,r),Le(n,r));for(let o=0;o!==s;++o)e.setTextureCube(t[o]||_h,r[o])}function yg(i,t,e){let n=this.cache,s=t.length,r=ro(e,s);Ie(n,r)||(i.uniform1iv(this.addr,r),Le(n,r));for(let o=0;o!==s;++o)e.setTexture2DArray(t[o]||mh,r[o])}function vg(i){switch(i){case 5126:return eg;case 35664:return ng;case 35665:return ig;case 35666:return sg;case 35674:return rg;case 35675:return og;case 35676:return ag;case 5124:case 35670:return cg;case 35667:case 35671:return lg;case 35668:case 35672:return hg;case 35669:case 35673:return ug;case 5125:return dg;case 36294:return fg;case 36295:return pg;case 36296:return mg;case 35678:case 36198:case 36298:case 36306:case 35682:return gg;case 35679:case 36299:case 36307:return _g;case 35680:case 36300:case 36308:case 36293:return xg;case 36289:case 36303:case 36311:case 36292:return yg}}var Ha=class{constructor(t,e,n){this.id=t,this.addr=n,this.cache=[],this.type=e.type,this.setValue=tg(e.type)}},Va=class{constructor(t,e,n){this.id=t,this.addr=n,this.cache=[],this.type=e.type,this.size=e.size,this.setValue=vg(e.type)}},Ga=class{constructor(t){this.id=t,this.seq=[],this.map={}}setValue(t,e,n){let s=this.seq;for(let r=0,o=s.length;r!==o;++r){let a=s[r];a.setValue(t,e[a.id],n)}}},qo=/(\w+)(\])?(\[|\.)?/g;function Il(i,t){i.seq.push(t),i.map[t.id]=t}function Mg(i,t,e){let n=i.name,s=n.length;for(qo.lastIndex=0;;){let r=qo.exec(n),o=qo.lastIndex,a=r[1],c=r[2]==="]",l=r[3];if(c&&(a=a|0),l===void 0||l==="["&&o+2===s){Il(e,l===void 0?new Ha(a,i,t):new Va(a,i,t));break}else{let u=e.map[a];u===void 0&&(u=new Ga(a),Il(e,u)),e=u}}}var ss=class{constructor(t,e){this.seq=[],this.map={};let n=t.getProgramParameter(e,t.ACTIVE_UNIFORMS);for(let s=0;s<n;++s){let r=t.getActiveUniform(e,s),o=t.getUniformLocation(e,r.name);Mg(r,o,this)}}setValue(t,e,n,s){let r=this.map[e];r!==void 0&&r.setValue(t,n,s)}setOptional(t,e,n){let s=e[n];s!==void 0&&this.setValue(t,n,s)}static upload(t,e,n,s){for(let r=0,o=e.length;r!==o;++r){let a=e[r],c=n[a.id];c.needsUpdate!==!1&&a.setValue(t,c.value,s)}}static seqWithValue(t,e){let n=[];for(let s=0,r=t.length;s!==r;++s){let o=t[s];o.id in e&&n.push(o)}return n}};function Ll(i,t,e){let n=i.createShader(t);return i.shaderSource(n,e),i.compileShader(n),n}var Sg=37297,bg=0;function Eg(i,t){let e=i.split(`
`),n=[],s=Math.max(t-6,0),r=Math.min(t+6,e.length);for(let o=s;o<r;o++){let a=o+1;n.push(`${a===t?">":" "} ${a}: ${e[o]}`)}return n.join(`
`)}var Dl=new ee;function Tg(i){he._getMatrix(Dl,he.workingColorSpace,i);let t=`mat3( ${Dl.elements.map(e=>e.toFixed(4))} )`;switch(he.getTransfer(i)){case io:return[t,"LinearTransferOETF"];case xe:return[t,"sRGBTransferOETF"];default:return console.warn("THREE.WebGLProgram: Unsupported color space: ",i),[t,"LinearTransferOETF"]}}function Ul(i,t,e){let n=i.getShaderParameter(t,i.COMPILE_STATUS),s=i.getShaderInfoLog(t).trim();if(n&&s==="")return"";let r=/ERROR: 0:(\d+)/.exec(s);if(r){let o=parseInt(r[1]);return e.toUpperCase()+`

`+s+`

`+Eg(i.getShaderSource(t),o)}else return s}function wg(i,t){let e=Tg(t);return[`vec4 ${i}( vec4 value ) {`,`	return ${e[1]}( vec4( value.rgb * ${e[0]}, value.a ) );`,"}"].join(`
`)}function Ag(i,t){let e;switch(t){case Bu:e="Linear";break;case zu:e="Reinhard";break;case ku:e="Cineon";break;case pc:e="ACESFilmic";break;case Vu:e="AgX";break;case Gu:e="Neutral";break;case Hu:e="Custom";break;default:console.warn("THREE.WebGLProgram: Unsupported toneMapping:",t),e="Linear"}return"vec3 "+i+"( vec3 color ) { return "+e+"ToneMapping( color ); }"}var xr=new U;function Cg(){he.getLuminanceCoefficients(xr);let i=xr.x.toFixed(4),t=xr.y.toFixed(4),e=xr.z.toFixed(4);return["float luminance( const in vec3 rgb ) {",`	const vec3 weights = vec3( ${i}, ${t}, ${e} );`,"	return dot( weights, rgb );","}"].join(`
`)}function Rg(i){return[i.extensionClipCullDistance?"#extension GL_ANGLE_clip_cull_distance : require":"",i.extensionMultiDraw?"#extension GL_ANGLE_multi_draw : require":""].filter(Us).join(`
`)}function Pg(i){let t=[];for(let e in i){let n=i[e];n!==!1&&t.push("#define "+e+" "+n)}return t.join(`
`)}function Ig(i,t){let e={},n=i.getProgramParameter(t,i.ACTIVE_ATTRIBUTES);for(let s=0;s<n;s++){let r=i.getActiveAttrib(t,s),o=r.name,a=1;r.type===i.FLOAT_MAT2&&(a=2),r.type===i.FLOAT_MAT3&&(a=3),r.type===i.FLOAT_MAT4&&(a=4),e[o]={type:r.type,location:i.getAttribLocation(t,o),locationSize:a}}return e}function Us(i){return i!==""}function Nl(i,t){let e=t.numSpotLightShadows+t.numSpotLightMaps-t.numSpotLightShadowsWithMaps;return i.replace(/NUM_DIR_LIGHTS/g,t.numDirLights).replace(/NUM_SPOT_LIGHTS/g,t.numSpotLights).replace(/NUM_SPOT_LIGHT_MAPS/g,t.numSpotLightMaps).replace(/NUM_SPOT_LIGHT_COORDS/g,e).replace(/NUM_RECT_AREA_LIGHTS/g,t.numRectAreaLights).replace(/NUM_POINT_LIGHTS/g,t.numPointLights).replace(/NUM_HEMI_LIGHTS/g,t.numHemiLights).replace(/NUM_DIR_LIGHT_SHADOWS/g,t.numDirLightShadows).replace(/NUM_SPOT_LIGHT_SHADOWS_WITH_MAPS/g,t.numSpotLightShadowsWithMaps).replace(/NUM_SPOT_LIGHT_SHADOWS/g,t.numSpotLightShadows).replace(/NUM_POINT_LIGHT_SHADOWS/g,t.numPointLightShadows)}function Fl(i,t){return i.replace(/NUM_CLIPPING_PLANES/g,t.numClippingPlanes).replace(/UNION_CLIPPING_PLANES/g,t.numClippingPlanes-t.numClipIntersection)}var Lg=/^[ \t]*#include +<([\w\d./]+)>/gm;function Wa(i){return i.replace(Lg,Ug)}var Dg=new Map;function Ug(i,t){let e=ie[t];if(e===void 0){let n=Dg.get(t);if(n!==void 0)e=ie[n],console.warn('THREE.WebGLRenderer: Shader chunk "%s" has been deprecated. Use "%s" instead.',t,n);else throw new Error("Can not resolve #include <"+t+">")}return Wa(e)}var Ng=/#pragma unroll_loop_start\s+for\s*\(\s*int\s+i\s*=\s*(\d+)\s*;\s*i\s*<\s*(\d+)\s*;\s*i\s*\+\+\s*\)\s*{([\s\S]+?)}\s+#pragma unroll_loop_end/g;function Ol(i){return i.replace(Ng,Fg)}function Fg(i,t,e,n){let s="";for(let r=parseInt(t);r<parseInt(e);r++)s+=n.replace(/\[\s*i\s*\]/g,"[ "+r+" ]").replace(/UNROLLED_LOOP_INDEX/g,r);return s}function Bl(i){let t=`precision ${i.precision} float;
	precision ${i.precision} int;
	precision ${i.precision} sampler2D;
	precision ${i.precision} samplerCube;
	precision ${i.precision} sampler3D;
	precision ${i.precision} sampler2DArray;
	precision ${i.precision} sampler2DShadow;
	precision ${i.precision} samplerCubeShadow;
	precision ${i.precision} sampler2DArrayShadow;
	precision ${i.precision} isampler2D;
	precision ${i.precision} isampler3D;
	precision ${i.precision} isamplerCube;
	precision ${i.precision} isampler2DArray;
	precision ${i.precision} usampler2D;
	precision ${i.precision} usampler3D;
	precision ${i.precision} usamplerCube;
	precision ${i.precision} usampler2DArray;
	`;return i.precision==="highp"?t+=`
#define HIGH_PRECISION`:i.precision==="mediump"?t+=`
#define MEDIUM_PRECISION`:i.precision==="lowp"&&(t+=`
#define LOW_PRECISION`),t}function Og(i){let t="SHADOWMAP_TYPE_BASIC";return i.shadowMapType===jl?t="SHADOWMAP_TYPE_PCF":i.shadowMapType===dc?t="SHADOWMAP_TYPE_PCF_SOFT":i.shadowMapType===Yn&&(t="SHADOWMAP_TYPE_VSM"),t}function Bg(i){let t="ENVMAP_TYPE_CUBE";if(i.envMap)switch(i.envMapMode){case os:case as:t="ENVMAP_TYPE_CUBE";break;case no:t="ENVMAP_TYPE_CUBE_UV";break}return t}function zg(i){let t="ENVMAP_MODE_REFLECTION";if(i.envMap)switch(i.envMapMode){case as:t="ENVMAP_MODE_REFRACTION";break}return t}function kg(i){let t="ENVMAP_BLENDING_NONE";if(i.envMap)switch(i.combine){case fc:t="ENVMAP_BLENDING_MULTIPLY";break;case Fu:t="ENVMAP_BLENDING_MIX";break;case Ou:t="ENVMAP_BLENDING_ADD";break}return t}function Hg(i){let t=i.envMapCubeUVHeight;if(t===null)return null;let e=Math.log2(t)-2,n=1/t;return{texelWidth:1/(3*Math.max(Math.pow(2,e),7*16)),texelHeight:n,maxMip:e}}function Vg(i,t,e,n){let s=i.getContext(),r=e.defines,o=e.vertexShader,a=e.fragmentShader,c=Og(e),l=Bg(e),h=zg(e),u=kg(e),p=Hg(e),f=Rg(e),g=Pg(r),y=s.createProgram(),m,d,x=e.glslVersion?"#version "+e.glslVersion+`
`:"";e.isRawShaderMaterial?(m=["#define SHADER_TYPE "+e.shaderType,"#define SHADER_NAME "+e.shaderName,g].filter(Us).join(`
`),m.length>0&&(m+=`
`),d=["#define SHADER_TYPE "+e.shaderType,"#define SHADER_NAME "+e.shaderName,g].filter(Us).join(`
`),d.length>0&&(d+=`
`)):(m=[Bl(e),"#define SHADER_TYPE "+e.shaderType,"#define SHADER_NAME "+e.shaderName,g,e.extensionClipCullDistance?"#define USE_CLIP_DISTANCE":"",e.batching?"#define USE_BATCHING":"",e.batchingColor?"#define USE_BATCHING_COLOR":"",e.instancing?"#define USE_INSTANCING":"",e.instancingColor?"#define USE_INSTANCING_COLOR":"",e.instancingMorph?"#define USE_INSTANCING_MORPH":"",e.useFog&&e.fog?"#define USE_FOG":"",e.useFog&&e.fogExp2?"#define FOG_EXP2":"",e.map?"#define USE_MAP":"",e.envMap?"#define USE_ENVMAP":"",e.envMap?"#define "+h:"",e.lightMap?"#define USE_LIGHTMAP":"",e.aoMap?"#define USE_AOMAP":"",e.bumpMap?"#define USE_BUMPMAP":"",e.normalMap?"#define USE_NORMALMAP":"",e.normalMapObjectSpace?"#define USE_NORMALMAP_OBJECTSPACE":"",e.normalMapTangentSpace?"#define USE_NORMALMAP_TANGENTSPACE":"",e.displacementMap?"#define USE_DISPLACEMENTMAP":"",e.emissiveMap?"#define USE_EMISSIVEMAP":"",e.anisotropy?"#define USE_ANISOTROPY":"",e.anisotropyMap?"#define USE_ANISOTROPYMAP":"",e.clearcoatMap?"#define USE_CLEARCOATMAP":"",e.clearcoatRoughnessMap?"#define USE_CLEARCOAT_ROUGHNESSMAP":"",e.clearcoatNormalMap?"#define USE_CLEARCOAT_NORMALMAP":"",e.iridescenceMap?"#define USE_IRIDESCENCEMAP":"",e.iridescenceThicknessMap?"#define USE_IRIDESCENCE_THICKNESSMAP":"",e.specularMap?"#define USE_SPECULARMAP":"",e.specularColorMap?"#define USE_SPECULAR_COLORMAP":"",e.specularIntensityMap?"#define USE_SPECULAR_INTENSITYMAP":"",e.roughnessMap?"#define USE_ROUGHNESSMAP":"",e.metalnessMap?"#define USE_METALNESSMAP":"",e.alphaMap?"#define USE_ALPHAMAP":"",e.alphaHash?"#define USE_ALPHAHASH":"",e.transmission?"#define USE_TRANSMISSION":"",e.transmissionMap?"#define USE_TRANSMISSIONMAP":"",e.thicknessMap?"#define USE_THICKNESSMAP":"",e.sheenColorMap?"#define USE_SHEEN_COLORMAP":"",e.sheenRoughnessMap?"#define USE_SHEEN_ROUGHNESSMAP":"",e.mapUv?"#define MAP_UV "+e.mapUv:"",e.alphaMapUv?"#define ALPHAMAP_UV "+e.alphaMapUv:"",e.lightMapUv?"#define LIGHTMAP_UV "+e.lightMapUv:"",e.aoMapUv?"#define AOMAP_UV "+e.aoMapUv:"",e.emissiveMapUv?"#define EMISSIVEMAP_UV "+e.emissiveMapUv:"",e.bumpMapUv?"#define BUMPMAP_UV "+e.bumpMapUv:"",e.normalMapUv?"#define NORMALMAP_UV "+e.normalMapUv:"",e.displacementMapUv?"#define DISPLACEMENTMAP_UV "+e.displacementMapUv:"",e.metalnessMapUv?"#define METALNESSMAP_UV "+e.metalnessMapUv:"",e.roughnessMapUv?"#define ROUGHNESSMAP_UV "+e.roughnessMapUv:"",e.anisotropyMapUv?"#define ANISOTROPYMAP_UV "+e.anisotropyMapUv:"",e.clearcoatMapUv?"#define CLEARCOATMAP_UV "+e.clearcoatMapUv:"",e.clearcoatNormalMapUv?"#define CLEARCOAT_NORMALMAP_UV "+e.clearcoatNormalMapUv:"",e.clearcoatRoughnessMapUv?"#define CLEARCOAT_ROUGHNESSMAP_UV "+e.clearcoatRoughnessMapUv:"",e.iridescenceMapUv?"#define IRIDESCENCEMAP_UV "+e.iridescenceMapUv:"",e.iridescenceThicknessMapUv?"#define IRIDESCENCE_THICKNESSMAP_UV "+e.iridescenceThicknessMapUv:"",e.sheenColorMapUv?"#define SHEEN_COLORMAP_UV "+e.sheenColorMapUv:"",e.sheenRoughnessMapUv?"#define SHEEN_ROUGHNESSMAP_UV "+e.sheenRoughnessMapUv:"",e.specularMapUv?"#define SPECULARMAP_UV "+e.specularMapUv:"",e.specularColorMapUv?"#define SPECULAR_COLORMAP_UV "+e.specularColorMapUv:"",e.specularIntensityMapUv?"#define SPECULAR_INTENSITYMAP_UV "+e.specularIntensityMapUv:"",e.transmissionMapUv?"#define TRANSMISSIONMAP_UV "+e.transmissionMapUv:"",e.thicknessMapUv?"#define THICKNESSMAP_UV "+e.thicknessMapUv:"",e.vertexTangents&&e.flatShading===!1?"#define USE_TANGENT":"",e.vertexColors?"#define USE_COLOR":"",e.vertexAlphas?"#define USE_COLOR_ALPHA":"",e.vertexUv1s?"#define USE_UV1":"",e.vertexUv2s?"#define USE_UV2":"",e.vertexUv3s?"#define USE_UV3":"",e.pointsUvs?"#define USE_POINTS_UV":"",e.flatShading?"#define FLAT_SHADED":"",e.skinning?"#define USE_SKINNING":"",e.morphTargets?"#define USE_MORPHTARGETS":"",e.morphNormals&&e.flatShading===!1?"#define USE_MORPHNORMALS":"",e.morphColors?"#define USE_MORPHCOLORS":"",e.morphTargetsCount>0?"#define MORPHTARGETS_TEXTURE_STRIDE "+e.morphTextureStride:"",e.morphTargetsCount>0?"#define MORPHTARGETS_COUNT "+e.morphTargetsCount:"",e.doubleSided?"#define DOUBLE_SIDED":"",e.flipSided?"#define FLIP_SIDED":"",e.shadowMapEnabled?"#define USE_SHADOWMAP":"",e.shadowMapEnabled?"#define "+c:"",e.sizeAttenuation?"#define USE_SIZEATTENUATION":"",e.numLightProbes>0?"#define USE_LIGHT_PROBES":"",e.logarithmicDepthBuffer?"#define USE_LOGDEPTHBUF":"",e.reverseDepthBuffer?"#define USE_REVERSEDEPTHBUF":"","uniform mat4 modelMatrix;","uniform mat4 modelViewMatrix;","uniform mat4 projectionMatrix;","uniform mat4 viewMatrix;","uniform mat3 normalMatrix;","uniform vec3 cameraPosition;","uniform bool isOrthographic;","#ifdef USE_INSTANCING","	attribute mat4 instanceMatrix;","#endif","#ifdef USE_INSTANCING_COLOR","	attribute vec3 instanceColor;","#endif","#ifdef USE_INSTANCING_MORPH","	uniform sampler2D morphTexture;","#endif","attribute vec3 position;","attribute vec3 normal;","attribute vec2 uv;","#ifdef USE_UV1","	attribute vec2 uv1;","#endif","#ifdef USE_UV2","	attribute vec2 uv2;","#endif","#ifdef USE_UV3","	attribute vec2 uv3;","#endif","#ifdef USE_TANGENT","	attribute vec4 tangent;","#endif","#if defined( USE_COLOR_ALPHA )","	attribute vec4 color;","#elif defined( USE_COLOR )","	attribute vec3 color;","#endif","#ifdef USE_SKINNING","	attribute vec4 skinIndex;","	attribute vec4 skinWeight;","#endif",`
`].filter(Us).join(`
`),d=[Bl(e),"#define SHADER_TYPE "+e.shaderType,"#define SHADER_NAME "+e.shaderName,g,e.useFog&&e.fog?"#define USE_FOG":"",e.useFog&&e.fogExp2?"#define FOG_EXP2":"",e.alphaToCoverage?"#define ALPHA_TO_COVERAGE":"",e.map?"#define USE_MAP":"",e.matcap?"#define USE_MATCAP":"",e.envMap?"#define USE_ENVMAP":"",e.envMap?"#define "+l:"",e.envMap?"#define "+h:"",e.envMap?"#define "+u:"",p?"#define CUBEUV_TEXEL_WIDTH "+p.texelWidth:"",p?"#define CUBEUV_TEXEL_HEIGHT "+p.texelHeight:"",p?"#define CUBEUV_MAX_MIP "+p.maxMip+".0":"",e.lightMap?"#define USE_LIGHTMAP":"",e.aoMap?"#define USE_AOMAP":"",e.bumpMap?"#define USE_BUMPMAP":"",e.normalMap?"#define USE_NORMALMAP":"",e.normalMapObjectSpace?"#define USE_NORMALMAP_OBJECTSPACE":"",e.normalMapTangentSpace?"#define USE_NORMALMAP_TANGENTSPACE":"",e.emissiveMap?"#define USE_EMISSIVEMAP":"",e.anisotropy?"#define USE_ANISOTROPY":"",e.anisotropyMap?"#define USE_ANISOTROPYMAP":"",e.clearcoat?"#define USE_CLEARCOAT":"",e.clearcoatMap?"#define USE_CLEARCOATMAP":"",e.clearcoatRoughnessMap?"#define USE_CLEARCOAT_ROUGHNESSMAP":"",e.clearcoatNormalMap?"#define USE_CLEARCOAT_NORMALMAP":"",e.dispersion?"#define USE_DISPERSION":"",e.iridescence?"#define USE_IRIDESCENCE":"",e.iridescenceMap?"#define USE_IRIDESCENCEMAP":"",e.iridescenceThicknessMap?"#define USE_IRIDESCENCE_THICKNESSMAP":"",e.specularMap?"#define USE_SPECULARMAP":"",e.specularColorMap?"#define USE_SPECULAR_COLORMAP":"",e.specularIntensityMap?"#define USE_SPECULAR_INTENSITYMAP":"",e.roughnessMap?"#define USE_ROUGHNESSMAP":"",e.metalnessMap?"#define USE_METALNESSMAP":"",e.alphaMap?"#define USE_ALPHAMAP":"",e.alphaTest?"#define USE_ALPHATEST":"",e.alphaHash?"#define USE_ALPHAHASH":"",e.sheen?"#define USE_SHEEN":"",e.sheenColorMap?"#define USE_SHEEN_COLORMAP":"",e.sheenRoughnessMap?"#define USE_SHEEN_ROUGHNESSMAP":"",e.transmission?"#define USE_TRANSMISSION":"",e.transmissionMap?"#define USE_TRANSMISSIONMAP":"",e.thicknessMap?"#define USE_THICKNESSMAP":"",e.vertexTangents&&e.flatShading===!1?"#define USE_TANGENT":"",e.vertexColors||e.instancingColor||e.batchingColor?"#define USE_COLOR":"",e.vertexAlphas?"#define USE_COLOR_ALPHA":"",e.vertexUv1s?"#define USE_UV1":"",e.vertexUv2s?"#define USE_UV2":"",e.vertexUv3s?"#define USE_UV3":"",e.pointsUvs?"#define USE_POINTS_UV":"",e.gradientMap?"#define USE_GRADIENTMAP":"",e.flatShading?"#define FLAT_SHADED":"",e.doubleSided?"#define DOUBLE_SIDED":"",e.flipSided?"#define FLIP_SIDED":"",e.shadowMapEnabled?"#define USE_SHADOWMAP":"",e.shadowMapEnabled?"#define "+c:"",e.premultipliedAlpha?"#define PREMULTIPLIED_ALPHA":"",e.numLightProbes>0?"#define USE_LIGHT_PROBES":"",e.decodeVideoTexture?"#define DECODE_VIDEO_TEXTURE":"",e.decodeVideoTextureEmissive?"#define DECODE_VIDEO_TEXTURE_EMISSIVE":"",e.logarithmicDepthBuffer?"#define USE_LOGDEPTHBUF":"",e.reverseDepthBuffer?"#define USE_REVERSEDEPTHBUF":"","uniform mat4 viewMatrix;","uniform vec3 cameraPosition;","uniform bool isOrthographic;",e.toneMapping!==fi?"#define TONE_MAPPING":"",e.toneMapping!==fi?ie.tonemapping_pars_fragment:"",e.toneMapping!==fi?Ag("toneMapping",e.toneMapping):"",e.dithering?"#define DITHERING":"",e.opaque?"#define OPAQUE":"",ie.colorspace_pars_fragment,wg("linearToOutputTexel",e.outputColorSpace),Cg(),e.useDepthPacking?"#define DEPTH_PACKING "+e.depthPacking:"",`
`].filter(Us).join(`
`)),o=Wa(o),o=Nl(o,e),o=Fl(o,e),a=Wa(a),a=Nl(a,e),a=Fl(a,e),o=Ol(o),a=Ol(a),e.isRawShaderMaterial!==!0&&(x=`#version 300 es
`,m=[f,"#define attribute in","#define varying out","#define texture2D texture"].join(`
`)+`
`+m,d=["#define varying in",e.glslVersion===Jc?"":"layout(location = 0) out highp vec4 pc_fragColor;",e.glslVersion===Jc?"":"#define gl_FragColor pc_fragColor","#define gl_FragDepthEXT gl_FragDepth","#define texture2D texture","#define textureCube texture","#define texture2DProj textureProj","#define texture2DLodEXT textureLod","#define texture2DProjLodEXT textureProjLod","#define textureCubeLodEXT textureLod","#define texture2DGradEXT textureGrad","#define texture2DProjGradEXT textureProjGrad","#define textureCubeGradEXT textureGrad"].join(`
`)+`
`+d);let _=x+m+o,v=x+d+a,C=Ll(s,s.VERTEX_SHADER,_),w=Ll(s,s.FRAGMENT_SHADER,v);s.attachShader(y,C),s.attachShader(y,w),e.index0AttributeName!==void 0?s.bindAttribLocation(y,0,e.index0AttributeName):e.morphTargets===!0&&s.bindAttribLocation(y,0,"position"),s.linkProgram(y);function A(R){if(i.debug.checkShaderErrors){let F=s.getProgramInfoLog(y).trim(),V=s.getShaderInfoLog(C).trim(),z=s.getShaderInfoLog(w).trim(),k=!0,I=!0;if(s.getProgramParameter(y,s.LINK_STATUS)===!1)if(k=!1,typeof i.debug.onShaderError=="function")i.debug.onShaderError(s,y,C,w);else{let Y=Ul(s,C,"vertex"),B=Ul(s,w,"fragment");console.error("THREE.WebGLProgram: Shader Error "+s.getError()+" - VALIDATE_STATUS "+s.getProgramParameter(y,s.VALIDATE_STATUS)+`

Material Name: `+R.name+`
Material Type: `+R.type+`

Program Info Log: `+F+`
`+Y+`
`+B)}else F!==""?console.warn("THREE.WebGLProgram: Program Info Log:",F):(V===""||z==="")&&(I=!1);I&&(R.diagnostics={runnable:k,programLog:F,vertexShader:{log:V,prefix:m},fragmentShader:{log:z,prefix:d}})}s.deleteShader(C),s.deleteShader(w),P=new ss(s,y),E=Ig(s,y)}let P;this.getUniforms=function(){return P===void 0&&A(this),P};let E;this.getAttributes=function(){return E===void 0&&A(this),E};let M=e.rendererExtensionParallelShaderCompile===!1;return this.isReady=function(){return M===!1&&(M=s.getProgramParameter(y,Sg)),M},this.destroy=function(){n.releaseStatesOfProgram(this),s.deleteProgram(y),this.program=void 0},this.type=e.shaderType,this.name=e.shaderName,this.id=bg++,this.cacheKey=t,this.usedTimes=1,this.program=y,this.vertexShader=C,this.fragmentShader=w,this}var Gg=0,Xa=class{constructor(){this.shaderCache=new Map,this.materialCache=new Map}update(t){let e=t.vertexShader,n=t.fragmentShader,s=this._getShaderStage(e),r=this._getShaderStage(n),o=this._getShaderCacheForMaterial(t);return o.has(s)===!1&&(o.add(s),s.usedTimes++),o.has(r)===!1&&(o.add(r),r.usedTimes++),this}remove(t){let e=this.materialCache.get(t);for(let n of e)n.usedTimes--,n.usedTimes===0&&this.shaderCache.delete(n.code);return this.materialCache.delete(t),this}getVertexShaderID(t){return this._getShaderStage(t.vertexShader).id}getFragmentShaderID(t){return this._getShaderStage(t.fragmentShader).id}dispose(){this.shaderCache.clear(),this.materialCache.clear()}_getShaderCacheForMaterial(t){let e=this.materialCache,n=e.get(t);return n===void 0&&(n=new Set,e.set(t,n)),n}_getShaderStage(t){let e=this.shaderCache,n=e.get(t);return n===void 0&&(n=new Ya(t),e.set(t,n)),n}},Ya=class{constructor(t){this.id=Gg++,this.code=t,this.usedTimes=0}};function Wg(i,t,e,n,s,r,o){let a=new Fr,c=new Xa,l=new Set,h=[],u=s.logarithmicDepthBuffer,p=s.vertexTextures,f=s.precision,g={MeshDepthMaterial:"depth",MeshDistanceMaterial:"distanceRGBA",MeshNormalMaterial:"normal",MeshBasicMaterial:"basic",MeshLambertMaterial:"lambert",MeshPhongMaterial:"phong",MeshToonMaterial:"toon",MeshStandardMaterial:"physical",MeshPhysicalMaterial:"physical",MeshMatcapMaterial:"matcap",LineBasicMaterial:"basic",LineDashedMaterial:"dashed",PointsMaterial:"points",ShadowMaterial:"shadow",SpriteMaterial:"sprite"};function y(E){return l.add(E),E===0?"uv":`uv${E}`}function m(E,M,R,F,V){let z=F.fog,k=V.geometry,I=E.isMeshStandardMaterial?F.environment:null,Y=(E.isMeshStandardMaterial?e:t).get(E.envMap||I),B=Y&&Y.mapping===no?Y.image.height:null,Q=g[E.type];E.precision!==null&&(f=s.getMaxPrecision(E.precision),f!==E.precision&&console.warn("THREE.WebGLProgram.getParameters:",E.precision,"not supported, using",f,"instead."));let ut=k.morphAttributes.position||k.morphAttributes.normal||k.morphAttributes.color,Et=ut!==void 0?ut.length:0,zt=0;k.morphAttributes.position!==void 0&&(zt=1),k.morphAttributes.normal!==void 0&&(zt=2),k.morphAttributes.color!==void 0&&(zt=3);let $t,H,st,pt;if(Q){let nt=In[Q];$t=nt.vertexShader,H=nt.fragmentShader}else $t=E.vertexShader,H=E.fragmentShader,c.update(E),st=c.getVertexShaderID(E),pt=c.getFragmentShaderID(E);let ct=i.getRenderTarget(),Pt=i.state.buffers.depth.getReversed(),At=V.isInstancedMesh===!0,Ot=V.isBatchedMesh===!0,Xt=!!E.map,Jt=!!E.matcap,le=!!Y,N=!!E.aoMap,de=!!E.lightMap,Yt=!!E.bumpMap,qt=!!E.normalMap,Vt=!!E.displacementMap,Lt=!!E.emissiveMap,Ut=!!E.metalnessMap,T=!!E.roughnessMap,S=E.anisotropy>0,X=E.clearcoat>0,j=E.dispersion>0,at=E.iridescence>0,J=E.sheen>0,Ft=E.transmission>0,St=S&&!!E.anisotropyMap,Rt=X&&!!E.clearcoatMap,re=X&&!!E.clearcoatNormalMap,lt=X&&!!E.clearcoatRoughnessMap,K=at&&!!E.iridescenceMap,Nt=at&&!!E.iridescenceThicknessMap,Zt=J&&!!E.sheenColorMap,It=J&&!!E.sheenRoughnessMap,ne=!!E.specularMap,Kt=!!E.specularColorMap,Gt=!!E.specularIntensityMap,L=Ft&&!!E.transmissionMap,dt=Ft&&!!E.thicknessMap,q=!!E.gradientMap,tt=!!E.alphaMap,wt=E.alphaTest>0,bt=!!E.alphaHash,et=!!E.extensions,ot=fi;E.toneMapped&&(ct===null||ct.isXRRenderTarget===!0)&&(ot=i.toneMapping);let $={shaderID:Q,shaderType:E.type,shaderName:E.name,vertexShader:$t,fragmentShader:H,defines:E.defines,customVertexShaderID:st,customFragmentShaderID:pt,isRawShaderMaterial:E.isRawShaderMaterial===!0,glslVersion:E.glslVersion,precision:f,batching:Ot,batchingColor:Ot&&V._colorsTexture!==null,instancing:At,instancingColor:At&&V.instanceColor!==null,instancingMorph:At&&V.morphTexture!==null,supportsVertexTextures:p,outputColorSpace:ct===null?i.outputColorSpace:ct.isXRRenderTarget===!0?ct.texture.colorSpace:_s,alphaToCoverage:!!E.alphaToCoverage,map:Xt,matcap:Jt,envMap:le,envMapMode:le&&Y.mapping,envMapCubeUVHeight:B,aoMap:N,lightMap:de,bumpMap:Yt,normalMap:qt,displacementMap:p&&Vt,emissiveMap:Lt,normalMapObjectSpace:qt&&E.normalMapType===qu,normalMapTangentSpace:qt&&E.normalMapType===Mc,metalnessMap:Ut,roughnessMap:T,anisotropy:S,anisotropyMap:St,clearcoat:X,clearcoatMap:Rt,clearcoatNormalMap:re,clearcoatRoughnessMap:lt,dispersion:j,iridescence:at,iridescenceMap:K,iridescenceThicknessMap:Nt,sheen:J,sheenColorMap:Zt,sheenRoughnessMap:It,specularMap:ne,specularColorMap:Kt,specularIntensityMap:Gt,transmission:Ft,transmissionMap:L,thicknessMap:dt,gradientMap:q,opaque:E.transparent===!1&&E.blending===es&&E.alphaToCoverage===!1,alphaMap:tt,alphaTest:wt,alphaHash:bt,combine:E.combine,mapUv:Xt&&y(E.map.channel),aoMapUv:N&&y(E.aoMap.channel),lightMapUv:de&&y(E.lightMap.channel),bumpMapUv:Yt&&y(E.bumpMap.channel),normalMapUv:qt&&y(E.normalMap.channel),displacementMapUv:Vt&&y(E.displacementMap.channel),emissiveMapUv:Lt&&y(E.emissiveMap.channel),metalnessMapUv:Ut&&y(E.metalnessMap.channel),roughnessMapUv:T&&y(E.roughnessMap.channel),anisotropyMapUv:St&&y(E.anisotropyMap.channel),clearcoatMapUv:Rt&&y(E.clearcoatMap.channel),clearcoatNormalMapUv:re&&y(E.clearcoatNormalMap.channel),clearcoatRoughnessMapUv:lt&&y(E.clearcoatRoughnessMap.channel),iridescenceMapUv:K&&y(E.iridescenceMap.channel),iridescenceThicknessMapUv:Nt&&y(E.iridescenceThicknessMap.channel),sheenColorMapUv:Zt&&y(E.sheenColorMap.channel),sheenRoughnessMapUv:It&&y(E.sheenRoughnessMap.channel),specularMapUv:ne&&y(E.specularMap.channel),specularColorMapUv:Kt&&y(E.specularColorMap.channel),specularIntensityMapUv:Gt&&y(E.specularIntensityMap.channel),transmissionMapUv:L&&y(E.transmissionMap.channel),thicknessMapUv:dt&&y(E.thicknessMap.channel),alphaMapUv:tt&&y(E.alphaMap.channel),vertexTangents:!!k.attributes.tangent&&(qt||S),vertexColors:E.vertexColors,vertexAlphas:E.vertexColors===!0&&!!k.attributes.color&&k.attributes.color.itemSize===4,pointsUvs:V.isPoints===!0&&!!k.attributes.uv&&(Xt||tt),fog:!!z,useFog:E.fog===!0,fogExp2:!!z&&z.isFogExp2,flatShading:E.flatShading===!0,sizeAttenuation:E.sizeAttenuation===!0,logarithmicDepthBuffer:u,reverseDepthBuffer:Pt,skinning:V.isSkinnedMesh===!0,morphTargets:k.morphAttributes.position!==void 0,morphNormals:k.morphAttributes.normal!==void 0,morphColors:k.morphAttributes.color!==void 0,morphTargetsCount:Et,morphTextureStride:zt,numDirLights:M.directional.length,numPointLights:M.point.length,numSpotLights:M.spot.length,numSpotLightMaps:M.spotLightMap.length,numRectAreaLights:M.rectArea.length,numHemiLights:M.hemi.length,numDirLightShadows:M.directionalShadowMap.length,numPointLightShadows:M.pointShadowMap.length,numSpotLightShadows:M.spotShadowMap.length,numSpotLightShadowsWithMaps:M.numSpotLightShadowsWithMaps,numLightProbes:M.numLightProbes,numClippingPlanes:o.numPlanes,numClipIntersection:o.numIntersection,dithering:E.dithering,shadowMapEnabled:i.shadowMap.enabled&&R.length>0,shadowMapType:i.shadowMap.type,toneMapping:ot,decodeVideoTexture:Xt&&E.map.isVideoTexture===!0&&he.getTransfer(E.map.colorSpace)===xe,decodeVideoTextureEmissive:Lt&&E.emissiveMap.isVideoTexture===!0&&he.getTransfer(E.emissiveMap.colorSpace)===xe,premultipliedAlpha:E.premultipliedAlpha,doubleSided:E.side===en,flipSided:E.side===ke,useDepthPacking:E.depthPacking>=0,depthPacking:E.depthPacking||0,index0AttributeName:E.index0AttributeName,extensionClipCullDistance:et&&E.extensions.clipCullDistance===!0&&n.has("WEBGL_clip_cull_distance"),extensionMultiDraw:(et&&E.extensions.multiDraw===!0||Ot)&&n.has("WEBGL_multi_draw"),rendererExtensionParallelShaderCompile:n.has("KHR_parallel_shader_compile"),customProgramCacheKey:E.customProgramCacheKey()};return $.vertexUv1s=l.has(1),$.vertexUv2s=l.has(2),$.vertexUv3s=l.has(3),l.clear(),$}function d(E){let M=[];if(E.shaderID?M.push(E.shaderID):(M.push(E.customVertexShaderID),M.push(E.customFragmentShaderID)),E.defines!==void 0)for(let R in E.defines)M.push(R),M.push(E.defines[R]);return E.isRawShaderMaterial===!1&&(x(M,E),_(M,E),M.push(i.outputColorSpace)),M.push(E.customProgramCacheKey),M.join()}function x(E,M){E.push(M.precision),E.push(M.outputColorSpace),E.push(M.envMapMode),E.push(M.envMapCubeUVHeight),E.push(M.mapUv),E.push(M.alphaMapUv),E.push(M.lightMapUv),E.push(M.aoMapUv),E.push(M.bumpMapUv),E.push(M.normalMapUv),E.push(M.displacementMapUv),E.push(M.emissiveMapUv),E.push(M.metalnessMapUv),E.push(M.roughnessMapUv),E.push(M.anisotropyMapUv),E.push(M.clearcoatMapUv),E.push(M.clearcoatNormalMapUv),E.push(M.clearcoatRoughnessMapUv),E.push(M.iridescenceMapUv),E.push(M.iridescenceThicknessMapUv),E.push(M.sheenColorMapUv),E.push(M.sheenRoughnessMapUv),E.push(M.specularMapUv),E.push(M.specularColorMapUv),E.push(M.specularIntensityMapUv),E.push(M.transmissionMapUv),E.push(M.thicknessMapUv),E.push(M.combine),E.push(M.fogExp2),E.push(M.sizeAttenuation),E.push(M.morphTargetsCount),E.push(M.morphAttributeCount),E.push(M.numDirLights),E.push(M.numPointLights),E.push(M.numSpotLights),E.push(M.numSpotLightMaps),E.push(M.numHemiLights),E.push(M.numRectAreaLights),E.push(M.numDirLightShadows),E.push(M.numPointLightShadows),E.push(M.numSpotLightShadows),E.push(M.numSpotLightShadowsWithMaps),E.push(M.numLightProbes),E.push(M.shadowMapType),E.push(M.toneMapping),E.push(M.numClippingPlanes),E.push(M.numClipIntersection),E.push(M.depthPacking)}function _(E,M){a.disableAll(),M.supportsVertexTextures&&a.enable(0),M.instancing&&a.enable(1),M.instancingColor&&a.enable(2),M.instancingMorph&&a.enable(3),M.matcap&&a.enable(4),M.envMap&&a.enable(5),M.normalMapObjectSpace&&a.enable(6),M.normalMapTangentSpace&&a.enable(7),M.clearcoat&&a.enable(8),M.iridescence&&a.enable(9),M.alphaTest&&a.enable(10),M.vertexColors&&a.enable(11),M.vertexAlphas&&a.enable(12),M.vertexUv1s&&a.enable(13),M.vertexUv2s&&a.enable(14),M.vertexUv3s&&a.enable(15),M.vertexTangents&&a.enable(16),M.anisotropy&&a.enable(17),M.alphaHash&&a.enable(18),M.batching&&a.enable(19),M.dispersion&&a.enable(20),M.batchingColor&&a.enable(21),E.push(a.mask),a.disableAll(),M.fog&&a.enable(0),M.useFog&&a.enable(1),M.flatShading&&a.enable(2),M.logarithmicDepthBuffer&&a.enable(3),M.reverseDepthBuffer&&a.enable(4),M.skinning&&a.enable(5),M.morphTargets&&a.enable(6),M.morphNormals&&a.enable(7),M.morphColors&&a.enable(8),M.premultipliedAlpha&&a.enable(9),M.shadowMapEnabled&&a.enable(10),M.doubleSided&&a.enable(11),M.flipSided&&a.enable(12),M.useDepthPacking&&a.enable(13),M.dithering&&a.enable(14),M.transmission&&a.enable(15),M.sheen&&a.enable(16),M.opaque&&a.enable(17),M.pointsUvs&&a.enable(18),M.decodeVideoTexture&&a.enable(19),M.decodeVideoTextureEmissive&&a.enable(20),M.alphaToCoverage&&a.enable(21),E.push(a.mask)}function v(E){let M=g[E.type],R;if(M){let F=In[M];R=xs.clone(F.uniforms)}else R=E.uniforms;return R}function C(E,M){let R;for(let F=0,V=h.length;F<V;F++){let z=h[F];if(z.cacheKey===M){R=z,++R.usedTimes;break}}return R===void 0&&(R=new Vg(i,M,E,r),h.push(R)),R}function w(E){if(--E.usedTimes===0){let M=h.indexOf(E);h[M]=h[h.length-1],h.pop(),E.destroy()}}function A(E){c.remove(E)}function P(){c.dispose()}return{getParameters:m,getProgramCacheKey:d,getUniforms:v,acquireProgram:C,releaseProgram:w,releaseShaderCache:A,programs:h,dispose:P}}function Xg(){let i=new WeakMap;function t(o){return i.has(o)}function e(o){let a=i.get(o);return a===void 0&&(a={},i.set(o,a)),a}function n(o){i.delete(o)}function s(o,a,c){i.get(o)[a]=c}function r(){i=new WeakMap}return{has:t,get:e,remove:n,update:s,dispose:r}}function Yg(i,t){return i.groupOrder!==t.groupOrder?i.groupOrder-t.groupOrder:i.renderOrder!==t.renderOrder?i.renderOrder-t.renderOrder:i.material.id!==t.material.id?i.material.id-t.material.id:i.z!==t.z?i.z-t.z:i.id-t.id}function zl(i,t){return i.groupOrder!==t.groupOrder?i.groupOrder-t.groupOrder:i.renderOrder!==t.renderOrder?i.renderOrder-t.renderOrder:i.z!==t.z?t.z-i.z:i.id-t.id}function kl(){let i=[],t=0,e=[],n=[],s=[];function r(){t=0,e.length=0,n.length=0,s.length=0}function o(u,p,f,g,y,m){let d=i[t];return d===void 0?(d={id:u.id,object:u,geometry:p,material:f,groupOrder:g,renderOrder:u.renderOrder,z:y,group:m},i[t]=d):(d.id=u.id,d.object=u,d.geometry=p,d.material=f,d.groupOrder=g,d.renderOrder=u.renderOrder,d.z=y,d.group=m),t++,d}function a(u,p,f,g,y,m){let d=o(u,p,f,g,y,m);f.transmission>0?n.push(d):f.transparent===!0?s.push(d):e.push(d)}function c(u,p,f,g,y,m){let d=o(u,p,f,g,y,m);f.transmission>0?n.unshift(d):f.transparent===!0?s.unshift(d):e.unshift(d)}function l(u,p){e.length>1&&e.sort(u||Yg),n.length>1&&n.sort(p||zl),s.length>1&&s.sort(p||zl)}function h(){for(let u=t,p=i.length;u<p;u++){let f=i[u];if(f.id===null)break;f.id=null,f.object=null,f.geometry=null,f.material=null,f.group=null}}return{opaque:e,transmissive:n,transparent:s,init:r,push:a,unshift:c,finish:h,sort:l}}function qg(){let i=new WeakMap;function t(n,s){let r=i.get(n),o;return r===void 0?(o=new kl,i.set(n,[o])):s>=r.length?(o=new kl,r.push(o)):o=r[s],o}function e(){i=new WeakMap}return{get:t,dispose:e}}function $g(){let i={};return{get:function(t){if(i[t.id]!==void 0)return i[t.id];let e;switch(t.type){case"DirectionalLight":e={direction:new U,color:new _t};break;case"SpotLight":e={position:new U,direction:new U,color:new _t,distance:0,coneCos:0,penumbraCos:0,decay:0};break;case"PointLight":e={position:new U,color:new _t,distance:0,decay:0};break;case"HemisphereLight":e={direction:new U,skyColor:new _t,groundColor:new _t};break;case"RectAreaLight":e={color:new _t,position:new U,halfWidth:new U,halfHeight:new U};break}return i[t.id]=e,e}}}function Zg(){let i={};return{get:function(t){if(i[t.id]!==void 0)return i[t.id];let e;switch(t.type){case"DirectionalLight":e={shadowIntensity:1,shadowBias:0,shadowNormalBias:0,shadowRadius:1,shadowMapSize:new Wt};break;case"SpotLight":e={shadowIntensity:1,shadowBias:0,shadowNormalBias:0,shadowRadius:1,shadowMapSize:new Wt};break;case"PointLight":e={shadowIntensity:1,shadowBias:0,shadowNormalBias:0,shadowRadius:1,shadowMapSize:new Wt,shadowCameraNear:1,shadowCameraFar:1e3};break}return i[t.id]=e,e}}}var Kg=0;function Jg(i,t){return(t.castShadow?2:0)-(i.castShadow?2:0)+(t.map?1:0)-(i.map?1:0)}function jg(i){let t=new $g,e=Zg(),n={version:0,hash:{directionalLength:-1,pointLength:-1,spotLength:-1,rectAreaLength:-1,hemiLength:-1,numDirectionalShadows:-1,numPointShadows:-1,numSpotShadows:-1,numSpotMaps:-1,numLightProbes:-1},ambient:[0,0,0],probe:[],directional:[],directionalShadow:[],directionalShadowMap:[],directionalShadowMatrix:[],spot:[],spotLightMap:[],spotShadow:[],spotShadowMap:[],spotLightMatrix:[],rectArea:[],rectAreaLTC1:null,rectAreaLTC2:null,point:[],pointShadow:[],pointShadowMap:[],pointShadowMatrix:[],hemi:[],numSpotLightShadowsWithMaps:0,numLightProbes:0};for(let l=0;l<9;l++)n.probe.push(new U);let s=new U,r=new ce,o=new ce;function a(l){let h=0,u=0,p=0;for(let E=0;E<9;E++)n.probe[E].set(0,0,0);let f=0,g=0,y=0,m=0,d=0,x=0,_=0,v=0,C=0,w=0,A=0;l.sort(Jg);for(let E=0,M=l.length;E<M;E++){let R=l[E],F=R.color,V=R.intensity,z=R.distance,k=R.shadow&&R.shadow.map?R.shadow.map.texture:null;if(R.isAmbientLight)h+=F.r*V,u+=F.g*V,p+=F.b*V;else if(R.isLightProbe){for(let I=0;I<9;I++)n.probe[I].addScaledVector(R.sh.coefficients[I],V);A++}else if(R.isDirectionalLight){let I=t.get(R);if(I.color.copy(R.color).multiplyScalar(R.intensity),R.castShadow){let Y=R.shadow,B=e.get(R);B.shadowIntensity=Y.intensity,B.shadowBias=Y.bias,B.shadowNormalBias=Y.normalBias,B.shadowRadius=Y.radius,B.shadowMapSize=Y.mapSize,n.directionalShadow[f]=B,n.directionalShadowMap[f]=k,n.directionalShadowMatrix[f]=R.shadow.matrix,x++}n.directional[f]=I,f++}else if(R.isSpotLight){let I=t.get(R);I.position.setFromMatrixPosition(R.matrixWorld),I.color.copy(F).multiplyScalar(V),I.distance=z,I.coneCos=Math.cos(R.angle),I.penumbraCos=Math.cos(R.angle*(1-R.penumbra)),I.decay=R.decay,n.spot[y]=I;let Y=R.shadow;if(R.map&&(n.spotLightMap[C]=R.map,C++,Y.updateMatrices(R),R.castShadow&&w++),n.spotLightMatrix[y]=Y.matrix,R.castShadow){let B=e.get(R);B.shadowIntensity=Y.intensity,B.shadowBias=Y.bias,B.shadowNormalBias=Y.normalBias,B.shadowRadius=Y.radius,B.shadowMapSize=Y.mapSize,n.spotShadow[y]=B,n.spotShadowMap[y]=k,v++}y++}else if(R.isRectAreaLight){let I=t.get(R);I.color.copy(F).multiplyScalar(V),I.halfWidth.set(R.width*.5,0,0),I.halfHeight.set(0,R.height*.5,0),n.rectArea[m]=I,m++}else if(R.isPointLight){let I=t.get(R);if(I.color.copy(R.color).multiplyScalar(R.intensity),I.distance=R.distance,I.decay=R.decay,R.castShadow){let Y=R.shadow,B=e.get(R);B.shadowIntensity=Y.intensity,B.shadowBias=Y.bias,B.shadowNormalBias=Y.normalBias,B.shadowRadius=Y.radius,B.shadowMapSize=Y.mapSize,B.shadowCameraNear=Y.camera.near,B.shadowCameraFar=Y.camera.far,n.pointShadow[g]=B,n.pointShadowMap[g]=k,n.pointShadowMatrix[g]=R.shadow.matrix,_++}n.point[g]=I,g++}else if(R.isHemisphereLight){let I=t.get(R);I.skyColor.copy(R.color).multiplyScalar(V),I.groundColor.copy(R.groundColor).multiplyScalar(V),n.hemi[d]=I,d++}}m>0&&(i.has("OES_texture_float_linear")===!0?(n.rectAreaLTC1=Mt.LTC_FLOAT_1,n.rectAreaLTC2=Mt.LTC_FLOAT_2):(n.rectAreaLTC1=Mt.LTC_HALF_1,n.rectAreaLTC2=Mt.LTC_HALF_2)),n.ambient[0]=h,n.ambient[1]=u,n.ambient[2]=p;let P=n.hash;(P.directionalLength!==f||P.pointLength!==g||P.spotLength!==y||P.rectAreaLength!==m||P.hemiLength!==d||P.numDirectionalShadows!==x||P.numPointShadows!==_||P.numSpotShadows!==v||P.numSpotMaps!==C||P.numLightProbes!==A)&&(n.directional.length=f,n.spot.length=y,n.rectArea.length=m,n.point.length=g,n.hemi.length=d,n.directionalShadow.length=x,n.directionalShadowMap.length=x,n.pointShadow.length=_,n.pointShadowMap.length=_,n.spotShadow.length=v,n.spotShadowMap.length=v,n.directionalShadowMatrix.length=x,n.pointShadowMatrix.length=_,n.spotLightMatrix.length=v+C-w,n.spotLightMap.length=C,n.numSpotLightShadowsWithMaps=w,n.numLightProbes=A,P.directionalLength=f,P.pointLength=g,P.spotLength=y,P.rectAreaLength=m,P.hemiLength=d,P.numDirectionalShadows=x,P.numPointShadows=_,P.numSpotShadows=v,P.numSpotMaps=C,P.numLightProbes=A,n.version=Kg++)}function c(l,h){let u=0,p=0,f=0,g=0,y=0,m=h.matrixWorldInverse;for(let d=0,x=l.length;d<x;d++){let _=l[d];if(_.isDirectionalLight){let v=n.directional[u];v.direction.setFromMatrixPosition(_.matrixWorld),s.setFromMatrixPosition(_.target.matrixWorld),v.direction.sub(s),v.direction.transformDirection(m),u++}else if(_.isSpotLight){let v=n.spot[f];v.position.setFromMatrixPosition(_.matrixWorld),v.position.applyMatrix4(m),v.direction.setFromMatrixPosition(_.matrixWorld),s.setFromMatrixPosition(_.target.matrixWorld),v.direction.sub(s),v.direction.transformDirection(m),f++}else if(_.isRectAreaLight){let v=n.rectArea[g];v.position.setFromMatrixPosition(_.matrixWorld),v.position.applyMatrix4(m),o.identity(),r.copy(_.matrixWorld),r.premultiply(m),o.extractRotation(r),v.halfWidth.set(_.width*.5,0,0),v.halfHeight.set(0,_.height*.5,0),v.halfWidth.applyMatrix4(o),v.halfHeight.applyMatrix4(o),g++}else if(_.isPointLight){let v=n.point[p];v.position.setFromMatrixPosition(_.matrixWorld),v.position.applyMatrix4(m),p++}else if(_.isHemisphereLight){let v=n.hemi[y];v.direction.setFromMatrixPosition(_.matrixWorld),v.direction.transformDirection(m),y++}}}return{setup:a,setupView:c,state:n}}function Hl(i){let t=new jg(i),e=[],n=[];function s(h){l.camera=h,e.length=0,n.length=0}function r(h){e.push(h)}function o(h){n.push(h)}function a(){t.setup(e)}function c(h){t.setupView(e,h)}let l={lightsArray:e,shadowsArray:n,camera:null,lights:t,transmissionRenderTarget:{}};return{init:s,state:l,setupLights:a,setupLightsView:c,pushLight:r,pushShadow:o}}function Qg(i){let t=new WeakMap;function e(s,r=0){let o=t.get(s),a;return o===void 0?(a=new Hl(i),t.set(s,[a])):r>=o.length?(a=new Hl(i),o.push(a)):a=o[r],a}function n(){t=new WeakMap}return{get:e,dispose:n}}var qa=class extends Nn{static get type(){return"MeshDepthMaterial"}constructor(t){super(),this.isMeshDepthMaterial=!0,this.depthPacking=Xu,this.map=null,this.alphaMap=null,this.displacementMap=null,this.displacementScale=1,this.displacementBias=0,this.wireframe=!1,this.wireframeLinewidth=1,this.setValues(t)}copy(t){return super.copy(t),this.depthPacking=t.depthPacking,this.map=t.map,this.alphaMap=t.alphaMap,this.displacementMap=t.displacementMap,this.displacementScale=t.displacementScale,this.displacementBias=t.displacementBias,this.wireframe=t.wireframe,this.wireframeLinewidth=t.wireframeLinewidth,this}},$a=class extends Nn{static get type(){return"MeshDistanceMaterial"}constructor(t){super(),this.isMeshDistanceMaterial=!0,this.map=null,this.alphaMap=null,this.displacementMap=null,this.displacementScale=1,this.displacementBias=0,this.setValues(t)}copy(t){return super.copy(t),this.map=t.map,this.alphaMap=t.alphaMap,this.displacementMap=t.displacementMap,this.displacementScale=t.displacementScale,this.displacementBias=t.displacementBias,this}},t0=`void main() {
	gl_Position = vec4( position, 1.0 );
}`,e0=`uniform sampler2D shadow_pass;
uniform vec2 resolution;
uniform float radius;
#include <packing>
void main() {
	const float samples = float( VSM_SAMPLES );
	float mean = 0.0;
	float squared_mean = 0.0;
	float uvStride = samples <= 1.0 ? 0.0 : 2.0 / ( samples - 1.0 );
	float uvStart = samples <= 1.0 ? 0.0 : - 1.0;
	for ( float i = 0.0; i < samples; i ++ ) {
		float uvOffset = uvStart + i * uvStride;
		#ifdef HORIZONTAL_PASS
			vec2 distribution = unpackRGBATo2Half( texture2D( shadow_pass, ( gl_FragCoord.xy + vec2( uvOffset, 0.0 ) * radius ) / resolution ) );
			mean += distribution.x;
			squared_mean += distribution.y * distribution.y + distribution.x * distribution.x;
		#else
			float depth = unpackRGBAToDepth( texture2D( shadow_pass, ( gl_FragCoord.xy + vec2( 0.0, uvOffset ) * radius ) / resolution ) );
			mean += depth;
			squared_mean += depth * depth;
		#endif
	}
	mean = mean / samples;
	squared_mean = squared_mean / samples;
	float std_dev = sqrt( squared_mean - mean * mean );
	gl_FragColor = pack2HalfToRGBA( vec2( mean, std_dev ) );
}`;function n0(i,t,e){let n=new ks,s=new Wt,r=new Wt,o=new be,a=new qa({depthPacking:Yu}),c=new $a,l={},h=e.maxTextureSize,u={[Dn]:ke,[ke]:Dn,[en]:en},p=new Pe({defines:{VSM_SAMPLES:8},uniforms:{shadow_pass:{value:null},resolution:{value:new Wt},radius:{value:4}},vertexShader:t0,fragmentShader:e0}),f=p.clone();f.defines.HORIZONTAL_PASS=1;let g=new Ge;g.setAttribute("position",new ve(new Float32Array([-1,-1,.5,3,-1,.5,-1,3,.5]),3));let y=new se(g,p),m=this;this.enabled=!1,this.autoUpdate=!0,this.needsUpdate=!1,this.type=jl;let d=this.type;this.render=function(w,A,P){if(m.enabled===!1||m.autoUpdate===!1&&m.needsUpdate===!1||w.length===0)return;let E=i.getRenderTarget(),M=i.getActiveCubeFace(),R=i.getActiveMipmapLevel(),F=i.state;F.setBlending(di),F.buffers.color.setClear(1,1,1,1),F.buffers.depth.setTest(!0),F.setScissorTest(!1);let V=d!==Yn&&this.type===Yn,z=d===Yn&&this.type!==Yn;for(let k=0,I=w.length;k<I;k++){let Y=w[k],B=Y.shadow;if(B===void 0){console.warn("THREE.WebGLShadowMap:",Y,"has no shadow.");continue}if(B.autoUpdate===!1&&B.needsUpdate===!1)continue;s.copy(B.mapSize);let Q=B.getFrameExtents();if(s.multiply(Q),r.copy(B.mapSize),(s.x>h||s.y>h)&&(s.x>h&&(r.x=Math.floor(h/Q.x),s.x=r.x*Q.x,B.mapSize.x=r.x),s.y>h&&(r.y=Math.floor(h/Q.y),s.y=r.y*Q.y,B.mapSize.y=r.y)),B.map===null||V===!0||z===!0){let Et=this.type!==Yn?{minFilter:fn,magFilter:fn}:{};B.map!==null&&B.map.dispose(),B.map=new En(s.x,s.y,Et),B.map.texture.name=Y.name+".shadowMap",B.camera.updateProjectionMatrix()}i.setRenderTarget(B.map),i.clear();let ut=B.getViewportCount();for(let Et=0;Et<ut;Et++){let zt=B.getViewport(Et);o.set(r.x*zt.x,r.y*zt.y,r.x*zt.z,r.y*zt.w),F.viewport(o),B.updateMatrices(Y,Et),n=B.getFrustum(),v(A,P,B.camera,Y,this.type)}B.isPointLightShadow!==!0&&this.type===Yn&&x(B,P),B.needsUpdate=!1}d=this.type,m.needsUpdate=!1,i.setRenderTarget(E,M,R)};function x(w,A){let P=t.update(y);p.defines.VSM_SAMPLES!==w.blurSamples&&(p.defines.VSM_SAMPLES=w.blurSamples,f.defines.VSM_SAMPLES=w.blurSamples,p.needsUpdate=!0,f.needsUpdate=!0),w.mapPass===null&&(w.mapPass=new En(s.x,s.y)),p.uniforms.shadow_pass.value=w.map.texture,p.uniforms.resolution.value=w.mapSize,p.uniforms.radius.value=w.radius,i.setRenderTarget(w.mapPass),i.clear(),i.renderBufferDirect(A,null,P,p,y,null),f.uniforms.shadow_pass.value=w.mapPass.texture,f.uniforms.resolution.value=w.mapSize,f.uniforms.radius.value=w.radius,i.setRenderTarget(w.map),i.clear(),i.renderBufferDirect(A,null,P,f,y,null)}function _(w,A,P,E){let M=null,R=P.isPointLight===!0?w.customDistanceMaterial:w.customDepthMaterial;if(R!==void 0)M=R;else if(M=P.isPointLight===!0?c:a,i.localClippingEnabled&&A.clipShadows===!0&&Array.isArray(A.clippingPlanes)&&A.clippingPlanes.length!==0||A.displacementMap&&A.displacementScale!==0||A.alphaMap&&A.alphaTest>0||A.map&&A.alphaTest>0){let F=M.uuid,V=A.uuid,z=l[F];z===void 0&&(z={},l[F]=z);let k=z[V];k===void 0&&(k=M.clone(),z[V]=k,A.addEventListener("dispose",C)),M=k}if(M.visible=A.visible,M.wireframe=A.wireframe,E===Yn?M.side=A.shadowSide!==null?A.shadowSide:A.side:M.side=A.shadowSide!==null?A.shadowSide:u[A.side],M.alphaMap=A.alphaMap,M.alphaTest=A.alphaTest,M.map=A.map,M.clipShadows=A.clipShadows,M.clippingPlanes=A.clippingPlanes,M.clipIntersection=A.clipIntersection,M.displacementMap=A.displacementMap,M.displacementScale=A.displacementScale,M.displacementBias=A.displacementBias,M.wireframeLinewidth=A.wireframeLinewidth,M.linewidth=A.linewidth,P.isPointLight===!0&&M.isMeshDistanceMaterial===!0){let F=i.properties.get(M);F.light=P}return M}function v(w,A,P,E,M){if(w.visible===!1)return;if(w.layers.test(A.layers)&&(w.isMesh||w.isLine||w.isPoints)&&(w.castShadow||w.receiveShadow&&M===Yn)&&(!w.frustumCulled||n.intersectsObject(w))){w.modelViewMatrix.multiplyMatrices(P.matrixWorldInverse,w.matrixWorld);let V=t.update(w),z=w.material;if(Array.isArray(z)){let k=V.groups;for(let I=0,Y=k.length;I<Y;I++){let B=k[I],Q=z[B.materialIndex];if(Q&&Q.visible){let ut=_(w,Q,E,M);w.onBeforeShadow(i,w,A,P,V,ut,B),i.renderBufferDirect(P,null,V,ut,w,B),w.onAfterShadow(i,w,A,P,V,ut,B)}}}else if(z.visible){let k=_(w,z,E,M);w.onBeforeShadow(i,w,A,P,V,k,null),i.renderBufferDirect(P,null,V,k,w,null),w.onAfterShadow(i,w,A,P,V,k,null)}}let F=w.children;for(let V=0,z=F.length;V<z;V++)v(F[V],A,P,E,M)}function C(w){w.target.removeEventListener("dispose",C);for(let P in l){let E=l[P],M=w.target.uuid;M in E&&(E[M].dispose(),delete E[M])}}}var i0={[jo]:Qo,[ta]:ia,[ea]:sa,[rs]:na,[Qo]:jo,[ia]:ta,[sa]:ea,[na]:rs};function s0(i,t){function e(){let L=!1,dt=new be,q=null,tt=new be(0,0,0,0);return{setMask:function(wt){q!==wt&&!L&&(i.colorMask(wt,wt,wt,wt),q=wt)},setLocked:function(wt){L=wt},setClear:function(wt,bt,et,ot,$){$===!0&&(wt*=ot,bt*=ot,et*=ot),dt.set(wt,bt,et,ot),tt.equals(dt)===!1&&(i.clearColor(wt,bt,et,ot),tt.copy(dt))},reset:function(){L=!1,q=null,tt.set(-1,0,0,0)}}}function n(){let L=!1,dt=!1,q=null,tt=null,wt=null;return{setReversed:function(bt){if(dt!==bt){let et=t.get("EXT_clip_control");dt?et.clipControlEXT(et.LOWER_LEFT_EXT,et.ZERO_TO_ONE_EXT):et.clipControlEXT(et.LOWER_LEFT_EXT,et.NEGATIVE_ONE_TO_ONE_EXT);let ot=wt;wt=null,this.setClear(ot)}dt=bt},getReversed:function(){return dt},setTest:function(bt){bt?ct(i.DEPTH_TEST):Pt(i.DEPTH_TEST)},setMask:function(bt){q!==bt&&!L&&(i.depthMask(bt),q=bt)},setFunc:function(bt){if(dt&&(bt=i0[bt]),tt!==bt){switch(bt){case jo:i.depthFunc(i.NEVER);break;case Qo:i.depthFunc(i.ALWAYS);break;case ta:i.depthFunc(i.LESS);break;case rs:i.depthFunc(i.LEQUAL);break;case ea:i.depthFunc(i.EQUAL);break;case na:i.depthFunc(i.GEQUAL);break;case ia:i.depthFunc(i.GREATER);break;case sa:i.depthFunc(i.NOTEQUAL);break;default:i.depthFunc(i.LEQUAL)}tt=bt}},setLocked:function(bt){L=bt},setClear:function(bt){wt!==bt&&(dt&&(bt=1-bt),i.clearDepth(bt),wt=bt)},reset:function(){L=!1,q=null,tt=null,wt=null,dt=!1}}}function s(){let L=!1,dt=null,q=null,tt=null,wt=null,bt=null,et=null,ot=null,$=null;return{setTest:function(nt){L||(nt?ct(i.STENCIL_TEST):Pt(i.STENCIL_TEST))},setMask:function(nt){dt!==nt&&!L&&(i.stencilMask(nt),dt=nt)},setFunc:function(nt,gt,xt){(q!==nt||tt!==gt||wt!==xt)&&(i.stencilFunc(nt,gt,xt),q=nt,tt=gt,wt=xt)},setOp:function(nt,gt,xt){(bt!==nt||et!==gt||ot!==xt)&&(i.stencilOp(nt,gt,xt),bt=nt,et=gt,ot=xt)},setLocked:function(nt){L=nt},setClear:function(nt){$!==nt&&(i.clearStencil(nt),$=nt)},reset:function(){L=!1,dt=null,q=null,tt=null,wt=null,bt=null,et=null,ot=null,$=null}}}let r=new e,o=new n,a=new s,c=new WeakMap,l=new WeakMap,h={},u={},p=new WeakMap,f=[],g=null,y=!1,m=null,d=null,x=null,_=null,v=null,C=null,w=null,A=new _t(0,0,0),P=0,E=!1,M=null,R=null,F=null,V=null,z=null,k=i.getParameter(i.MAX_COMBINED_TEXTURE_IMAGE_UNITS),I=!1,Y=0,B=i.getParameter(i.VERSION);B.indexOf("WebGL")!==-1?(Y=parseFloat(/^WebGL (\d)/.exec(B)[1]),I=Y>=1):B.indexOf("OpenGL ES")!==-1&&(Y=parseFloat(/^OpenGL ES (\d)/.exec(B)[1]),I=Y>=2);let Q=null,ut={},Et=i.getParameter(i.SCISSOR_BOX),zt=i.getParameter(i.VIEWPORT),$t=new be().fromArray(Et),H=new be().fromArray(zt);function st(L,dt,q,tt){let wt=new Uint8Array(4),bt=i.createTexture();i.bindTexture(L,bt),i.texParameteri(L,i.TEXTURE_MIN_FILTER,i.NEAREST),i.texParameteri(L,i.TEXTURE_MAG_FILTER,i.NEAREST);for(let et=0;et<q;et++)L===i.TEXTURE_3D||L===i.TEXTURE_2D_ARRAY?i.texImage3D(dt,0,i.RGBA,1,1,tt,0,i.RGBA,i.UNSIGNED_BYTE,wt):i.texImage2D(dt+et,0,i.RGBA,1,1,0,i.RGBA,i.UNSIGNED_BYTE,wt);return bt}let pt={};pt[i.TEXTURE_2D]=st(i.TEXTURE_2D,i.TEXTURE_2D,1),pt[i.TEXTURE_CUBE_MAP]=st(i.TEXTURE_CUBE_MAP,i.TEXTURE_CUBE_MAP_POSITIVE_X,6),pt[i.TEXTURE_2D_ARRAY]=st(i.TEXTURE_2D_ARRAY,i.TEXTURE_2D_ARRAY,1,1),pt[i.TEXTURE_3D]=st(i.TEXTURE_3D,i.TEXTURE_3D,1,1),r.setClear(0,0,0,1),o.setClear(1),a.setClear(0),ct(i.DEPTH_TEST),o.setFunc(rs),Yt(!1),qt(Wc),ct(i.CULL_FACE),N(di);function ct(L){h[L]!==!0&&(i.enable(L),h[L]=!0)}function Pt(L){h[L]!==!1&&(i.disable(L),h[L]=!1)}function At(L,dt){return u[L]!==dt?(i.bindFramebuffer(L,dt),u[L]=dt,L===i.DRAW_FRAMEBUFFER&&(u[i.FRAMEBUFFER]=dt),L===i.FRAMEBUFFER&&(u[i.DRAW_FRAMEBUFFER]=dt),!0):!1}function Ot(L,dt){let q=f,tt=!1;if(L){q=p.get(dt),q===void 0&&(q=[],p.set(dt,q));let wt=L.textures;if(q.length!==wt.length||q[0]!==i.COLOR_ATTACHMENT0){for(let bt=0,et=wt.length;bt<et;bt++)q[bt]=i.COLOR_ATTACHMENT0+bt;q.length=wt.length,tt=!0}}else q[0]!==i.BACK&&(q[0]=i.BACK,tt=!0);tt&&i.drawBuffers(q)}function Xt(L){return g!==L?(i.useProgram(L),g=L,!0):!1}let Jt={[wi]:i.FUNC_ADD,[yu]:i.FUNC_SUBTRACT,[vu]:i.FUNC_REVERSE_SUBTRACT};Jt[Mu]=i.MIN,Jt[Su]=i.MAX;let le={[bu]:i.ZERO,[Eu]:i.ONE,[Tu]:i.SRC_COLOR,[Ko]:i.SRC_ALPHA,[Iu]:i.SRC_ALPHA_SATURATE,[Ru]:i.DST_COLOR,[Au]:i.DST_ALPHA,[wu]:i.ONE_MINUS_SRC_COLOR,[Jo]:i.ONE_MINUS_SRC_ALPHA,[Pu]:i.ONE_MINUS_DST_COLOR,[Cu]:i.ONE_MINUS_DST_ALPHA,[Lu]:i.CONSTANT_COLOR,[Du]:i.ONE_MINUS_CONSTANT_COLOR,[Uu]:i.CONSTANT_ALPHA,[Nu]:i.ONE_MINUS_CONSTANT_ALPHA};function N(L,dt,q,tt,wt,bt,et,ot,$,nt){if(L===di){y===!0&&(Pt(i.BLEND),y=!1);return}if(y===!1&&(ct(i.BLEND),y=!0),L!==xu){if(L!==m||nt!==E){if((d!==wi||v!==wi)&&(i.blendEquation(i.FUNC_ADD),d=wi,v=wi),nt)switch(L){case es:i.blendFuncSeparate(i.ONE,i.ONE_MINUS_SRC_ALPHA,i.ONE,i.ONE_MINUS_SRC_ALPHA);break;case pi:i.blendFunc(i.ONE,i.ONE);break;case Xc:i.blendFuncSeparate(i.ZERO,i.ONE_MINUS_SRC_COLOR,i.ZERO,i.ONE);break;case Yc:i.blendFuncSeparate(i.ZERO,i.SRC_COLOR,i.ZERO,i.SRC_ALPHA);break;default:console.error("THREE.WebGLState: Invalid blending: ",L);break}else switch(L){case es:i.blendFuncSeparate(i.SRC_ALPHA,i.ONE_MINUS_SRC_ALPHA,i.ONE,i.ONE_MINUS_SRC_ALPHA);break;case pi:i.blendFunc(i.SRC_ALPHA,i.ONE);break;case Xc:i.blendFuncSeparate(i.ZERO,i.ONE_MINUS_SRC_COLOR,i.ZERO,i.ONE);break;case Yc:i.blendFunc(i.ZERO,i.SRC_COLOR);break;default:console.error("THREE.WebGLState: Invalid blending: ",L);break}x=null,_=null,C=null,w=null,A.set(0,0,0),P=0,m=L,E=nt}return}wt=wt||dt,bt=bt||q,et=et||tt,(dt!==d||wt!==v)&&(i.blendEquationSeparate(Jt[dt],Jt[wt]),d=dt,v=wt),(q!==x||tt!==_||bt!==C||et!==w)&&(i.blendFuncSeparate(le[q],le[tt],le[bt],le[et]),x=q,_=tt,C=bt,w=et),(ot.equals(A)===!1||$!==P)&&(i.blendColor(ot.r,ot.g,ot.b,$),A.copy(ot),P=$),m=L,E=!1}function de(L,dt){L.side===en?Pt(i.CULL_FACE):ct(i.CULL_FACE);let q=L.side===ke;dt&&(q=!q),Yt(q),L.blending===es&&L.transparent===!1?N(di):N(L.blending,L.blendEquation,L.blendSrc,L.blendDst,L.blendEquationAlpha,L.blendSrcAlpha,L.blendDstAlpha,L.blendColor,L.blendAlpha,L.premultipliedAlpha),o.setFunc(L.depthFunc),o.setTest(L.depthTest),o.setMask(L.depthWrite),r.setMask(L.colorWrite);let tt=L.stencilWrite;a.setTest(tt),tt&&(a.setMask(L.stencilWriteMask),a.setFunc(L.stencilFunc,L.stencilRef,L.stencilFuncMask),a.setOp(L.stencilFail,L.stencilZFail,L.stencilZPass)),Lt(L.polygonOffset,L.polygonOffsetFactor,L.polygonOffsetUnits),L.alphaToCoverage===!0?ct(i.SAMPLE_ALPHA_TO_COVERAGE):Pt(i.SAMPLE_ALPHA_TO_COVERAGE)}function Yt(L){M!==L&&(L?i.frontFace(i.CW):i.frontFace(i.CCW),M=L)}function qt(L){L!==gu?(ct(i.CULL_FACE),L!==R&&(L===Wc?i.cullFace(i.BACK):L===_u?i.cullFace(i.FRONT):i.cullFace(i.FRONT_AND_BACK))):Pt(i.CULL_FACE),R=L}function Vt(L){L!==F&&(I&&i.lineWidth(L),F=L)}function Lt(L,dt,q){L?(ct(i.POLYGON_OFFSET_FILL),(V!==dt||z!==q)&&(i.polygonOffset(dt,q),V=dt,z=q)):Pt(i.POLYGON_OFFSET_FILL)}function Ut(L){L?ct(i.SCISSOR_TEST):Pt(i.SCISSOR_TEST)}function T(L){L===void 0&&(L=i.TEXTURE0+k-1),Q!==L&&(i.activeTexture(L),Q=L)}function S(L,dt,q){q===void 0&&(Q===null?q=i.TEXTURE0+k-1:q=Q);let tt=ut[q];tt===void 0&&(tt={type:void 0,texture:void 0},ut[q]=tt),(tt.type!==L||tt.texture!==dt)&&(Q!==q&&(i.activeTexture(q),Q=q),i.bindTexture(L,dt||pt[L]),tt.type=L,tt.texture=dt)}function X(){let L=ut[Q];L!==void 0&&L.type!==void 0&&(i.bindTexture(L.type,null),L.type=void 0,L.texture=void 0)}function j(){try{i.compressedTexImage2D.apply(i,arguments)}catch(L){console.error("THREE.WebGLState:",L)}}function at(){try{i.compressedTexImage3D.apply(i,arguments)}catch(L){console.error("THREE.WebGLState:",L)}}function J(){try{i.texSubImage2D.apply(i,arguments)}catch(L){console.error("THREE.WebGLState:",L)}}function Ft(){try{i.texSubImage3D.apply(i,arguments)}catch(L){console.error("THREE.WebGLState:",L)}}function St(){try{i.compressedTexSubImage2D.apply(i,arguments)}catch(L){console.error("THREE.WebGLState:",L)}}function Rt(){try{i.compressedTexSubImage3D.apply(i,arguments)}catch(L){console.error("THREE.WebGLState:",L)}}function re(){try{i.texStorage2D.apply(i,arguments)}catch(L){console.error("THREE.WebGLState:",L)}}function lt(){try{i.texStorage3D.apply(i,arguments)}catch(L){console.error("THREE.WebGLState:",L)}}function K(){try{i.texImage2D.apply(i,arguments)}catch(L){console.error("THREE.WebGLState:",L)}}function Nt(){try{i.texImage3D.apply(i,arguments)}catch(L){console.error("THREE.WebGLState:",L)}}function Zt(L){$t.equals(L)===!1&&(i.scissor(L.x,L.y,L.z,L.w),$t.copy(L))}function It(L){H.equals(L)===!1&&(i.viewport(L.x,L.y,L.z,L.w),H.copy(L))}function ne(L,dt){let q=l.get(dt);q===void 0&&(q=new WeakMap,l.set(dt,q));let tt=q.get(L);tt===void 0&&(tt=i.getUniformBlockIndex(dt,L.name),q.set(L,tt))}function Kt(L,dt){let tt=l.get(dt).get(L);c.get(dt)!==tt&&(i.uniformBlockBinding(dt,tt,L.__bindingPointIndex),c.set(dt,tt))}function Gt(){i.disable(i.BLEND),i.disable(i.CULL_FACE),i.disable(i.DEPTH_TEST),i.disable(i.POLYGON_OFFSET_FILL),i.disable(i.SCISSOR_TEST),i.disable(i.STENCIL_TEST),i.disable(i.SAMPLE_ALPHA_TO_COVERAGE),i.blendEquation(i.FUNC_ADD),i.blendFunc(i.ONE,i.ZERO),i.blendFuncSeparate(i.ONE,i.ZERO,i.ONE,i.ZERO),i.blendColor(0,0,0,0),i.colorMask(!0,!0,!0,!0),i.clearColor(0,0,0,0),i.depthMask(!0),i.depthFunc(i.LESS),o.setReversed(!1),i.clearDepth(1),i.stencilMask(4294967295),i.stencilFunc(i.ALWAYS,0,4294967295),i.stencilOp(i.KEEP,i.KEEP,i.KEEP),i.clearStencil(0),i.cullFace(i.BACK),i.frontFace(i.CCW),i.polygonOffset(0,0),i.activeTexture(i.TEXTURE0),i.bindFramebuffer(i.FRAMEBUFFER,null),i.bindFramebuffer(i.DRAW_FRAMEBUFFER,null),i.bindFramebuffer(i.READ_FRAMEBUFFER,null),i.useProgram(null),i.lineWidth(1),i.scissor(0,0,i.canvas.width,i.canvas.height),i.viewport(0,0,i.canvas.width,i.canvas.height),h={},Q=null,ut={},u={},p=new WeakMap,f=[],g=null,y=!1,m=null,d=null,x=null,_=null,v=null,C=null,w=null,A=new _t(0,0,0),P=0,E=!1,M=null,R=null,F=null,V=null,z=null,$t.set(0,0,i.canvas.width,i.canvas.height),H.set(0,0,i.canvas.width,i.canvas.height),r.reset(),o.reset(),a.reset()}return{buffers:{color:r,depth:o,stencil:a},enable:ct,disable:Pt,bindFramebuffer:At,drawBuffers:Ot,useProgram:Xt,setBlending:N,setMaterial:de,setFlipSided:Yt,setCullFace:qt,setLineWidth:Vt,setPolygonOffset:Lt,setScissorTest:Ut,activeTexture:T,bindTexture:S,unbindTexture:X,compressedTexImage2D:j,compressedTexImage3D:at,texImage2D:K,texImage3D:Nt,updateUBOMapping:ne,uniformBlockBinding:Kt,texStorage2D:re,texStorage3D:lt,texSubImage2D:J,texSubImage3D:Ft,compressedTexSubImage2D:St,compressedTexSubImage3D:Rt,scissor:Zt,viewport:It,reset:Gt}}function Vl(i,t,e,n){let s=r0(n);switch(e){case ih:return i*t;case rh:return i*t;case oh:return i*t*2;case qs:return i*t/s.components*s.byteLength;case xc:return i*t/s.components*s.byteLength;case ah:return i*t*2/s.components*s.byteLength;case yc:return i*t*2/s.components*s.byteLength;case sh:return i*t*3/s.components*s.byteLength;case dn:return i*t*4/s.components*s.byteLength;case vc:return i*t*4/s.components*s.byteLength;case wr:case Ar:return Math.floor((i+3)/4)*Math.floor((t+3)/4)*8;case Cr:case Rr:return Math.floor((i+3)/4)*Math.floor((t+3)/4)*16;case la:case ua:return Math.max(i,16)*Math.max(t,8)/4;case ca:case ha:return Math.max(i,8)*Math.max(t,8)/2;case da:case fa:return Math.floor((i+3)/4)*Math.floor((t+3)/4)*8;case pa:return Math.floor((i+3)/4)*Math.floor((t+3)/4)*16;case ma:return Math.floor((i+3)/4)*Math.floor((t+3)/4)*16;case ga:return Math.floor((i+4)/5)*Math.floor((t+3)/4)*16;case _a:return Math.floor((i+4)/5)*Math.floor((t+4)/5)*16;case xa:return Math.floor((i+5)/6)*Math.floor((t+4)/5)*16;case ya:return Math.floor((i+5)/6)*Math.floor((t+5)/6)*16;case va:return Math.floor((i+7)/8)*Math.floor((t+4)/5)*16;case Ma:return Math.floor((i+7)/8)*Math.floor((t+5)/6)*16;case Sa:return Math.floor((i+7)/8)*Math.floor((t+7)/8)*16;case ba:return Math.floor((i+9)/10)*Math.floor((t+4)/5)*16;case Ea:return Math.floor((i+9)/10)*Math.floor((t+5)/6)*16;case Ta:return Math.floor((i+9)/10)*Math.floor((t+7)/8)*16;case wa:return Math.floor((i+9)/10)*Math.floor((t+9)/10)*16;case Aa:return Math.floor((i+11)/12)*Math.floor((t+9)/10)*16;case Ca:return Math.floor((i+11)/12)*Math.floor((t+11)/12)*16;case Pr:case Ra:case Pa:return Math.ceil(i/4)*Math.ceil(t/4)*16;case ch:case Ia:return Math.ceil(i/4)*Math.ceil(t/4)*8;case La:case Da:return Math.ceil(i/4)*Math.ceil(t/4)*16}throw new Error(`Unable to determine texture byte length for ${e} format.`)}function r0(i){switch(i){case Jn:case th:return{byteLength:1,components:1};case Bs:case eh:case Ys:return{byteLength:2,components:1};case gc:case _c:return{byteLength:2,components:4};case Ci:case mc:case _n:return{byteLength:4,components:1};case nh:return{byteLength:4,components:3}}throw new Error(`Unknown texture type ${i}.`)}function o0(i,t,e,n,s,r,o){let a=t.has("WEBGL_multisampled_render_to_texture")?t.get("WEBGL_multisampled_render_to_texture"):null,c=typeof navigator>"u"?!1:/OculusBrowser/g.test(navigator.userAgent),l=new Wt,h=new WeakMap,u,p=new WeakMap,f=!1;try{f=typeof OffscreenCanvas<"u"&&new OffscreenCanvas(1,1).getContext("2d")!==null}catch{}function g(T,S){return f?new OffscreenCanvas(T,S):Dr("canvas")}function y(T,S,X){let j=1,at=Ut(T);if((at.width>X||at.height>X)&&(j=X/Math.max(at.width,at.height)),j<1)if(typeof HTMLImageElement<"u"&&T instanceof HTMLImageElement||typeof HTMLCanvasElement<"u"&&T instanceof HTMLCanvasElement||typeof ImageBitmap<"u"&&T instanceof ImageBitmap||typeof VideoFrame<"u"&&T instanceof VideoFrame){let J=Math.floor(j*at.width),Ft=Math.floor(j*at.height);u===void 0&&(u=g(J,Ft));let St=S?g(J,Ft):u;return St.width=J,St.height=Ft,St.getContext("2d").drawImage(T,0,0,J,Ft),console.warn("THREE.WebGLRenderer: Texture has been resized from ("+at.width+"x"+at.height+") to ("+J+"x"+Ft+")."),St}else return"data"in T&&console.warn("THREE.WebGLRenderer: Image in DataTexture is too big ("+at.width+"x"+at.height+")."),T;return T}function m(T){return T.generateMipmaps}function d(T){i.generateMipmap(T)}function x(T){return T.isWebGLCubeRenderTarget?i.TEXTURE_CUBE_MAP:T.isWebGL3DRenderTarget?i.TEXTURE_3D:T.isWebGLArrayRenderTarget||T.isCompressedArrayTexture?i.TEXTURE_2D_ARRAY:i.TEXTURE_2D}function _(T,S,X,j,at=!1){if(T!==null){if(i[T]!==void 0)return i[T];console.warn("THREE.WebGLRenderer: Attempt to use non-existing WebGL internal format '"+T+"'")}let J=S;if(S===i.RED&&(X===i.FLOAT&&(J=i.R32F),X===i.HALF_FLOAT&&(J=i.R16F),X===i.UNSIGNED_BYTE&&(J=i.R8)),S===i.RED_INTEGER&&(X===i.UNSIGNED_BYTE&&(J=i.R8UI),X===i.UNSIGNED_SHORT&&(J=i.R16UI),X===i.UNSIGNED_INT&&(J=i.R32UI),X===i.BYTE&&(J=i.R8I),X===i.SHORT&&(J=i.R16I),X===i.INT&&(J=i.R32I)),S===i.RG&&(X===i.FLOAT&&(J=i.RG32F),X===i.HALF_FLOAT&&(J=i.RG16F),X===i.UNSIGNED_BYTE&&(J=i.RG8)),S===i.RG_INTEGER&&(X===i.UNSIGNED_BYTE&&(J=i.RG8UI),X===i.UNSIGNED_SHORT&&(J=i.RG16UI),X===i.UNSIGNED_INT&&(J=i.RG32UI),X===i.BYTE&&(J=i.RG8I),X===i.SHORT&&(J=i.RG16I),X===i.INT&&(J=i.RG32I)),S===i.RGB_INTEGER&&(X===i.UNSIGNED_BYTE&&(J=i.RGB8UI),X===i.UNSIGNED_SHORT&&(J=i.RGB16UI),X===i.UNSIGNED_INT&&(J=i.RGB32UI),X===i.BYTE&&(J=i.RGB8I),X===i.SHORT&&(J=i.RGB16I),X===i.INT&&(J=i.RGB32I)),S===i.RGBA_INTEGER&&(X===i.UNSIGNED_BYTE&&(J=i.RGBA8UI),X===i.UNSIGNED_SHORT&&(J=i.RGBA16UI),X===i.UNSIGNED_INT&&(J=i.RGBA32UI),X===i.BYTE&&(J=i.RGBA8I),X===i.SHORT&&(J=i.RGBA16I),X===i.INT&&(J=i.RGBA32I)),S===i.RGB&&X===i.UNSIGNED_INT_5_9_9_9_REV&&(J=i.RGB9_E5),S===i.RGBA){let Ft=at?io:he.getTransfer(j);X===i.FLOAT&&(J=i.RGBA32F),X===i.HALF_FLOAT&&(J=i.RGBA16F),X===i.UNSIGNED_BYTE&&(J=Ft===xe?i.SRGB8_ALPHA8:i.RGBA8),X===i.UNSIGNED_SHORT_4_4_4_4&&(J=i.RGBA4),X===i.UNSIGNED_SHORT_5_5_5_1&&(J=i.RGB5_A1)}return(J===i.R16F||J===i.R32F||J===i.RG16F||J===i.RG32F||J===i.RGBA16F||J===i.RGBA32F)&&t.get("EXT_color_buffer_float"),J}function v(T,S){let X;return T?S===null||S===Ci||S===cs?X=i.DEPTH24_STENCIL8:S===_n?X=i.DEPTH32F_STENCIL8:S===Bs&&(X=i.DEPTH24_STENCIL8,console.warn("DepthTexture: 16 bit depth attachment is not supported with stencil. Using 24-bit attachment.")):S===null||S===Ci||S===cs?X=i.DEPTH_COMPONENT24:S===_n?X=i.DEPTH_COMPONENT32F:S===Bs&&(X=i.DEPTH_COMPONENT16),X}function C(T,S){return m(T)===!0||T.isFramebufferTexture&&T.minFilter!==fn&&T.minFilter!==je?Math.log2(Math.max(S.width,S.height))+1:T.mipmaps!==void 0&&T.mipmaps.length>0?T.mipmaps.length:T.isCompressedTexture&&Array.isArray(T.image)?S.mipmaps.length:1}function w(T){let S=T.target;S.removeEventListener("dispose",w),P(S),S.isVideoTexture&&h.delete(S)}function A(T){let S=T.target;S.removeEventListener("dispose",A),M(S)}function P(T){let S=n.get(T);if(S.__webglInit===void 0)return;let X=T.source,j=p.get(X);if(j){let at=j[S.__cacheKey];at.usedTimes--,at.usedTimes===0&&E(T),Object.keys(j).length===0&&p.delete(X)}n.remove(T)}function E(T){let S=n.get(T);i.deleteTexture(S.__webglTexture);let X=T.source,j=p.get(X);delete j[S.__cacheKey],o.memory.textures--}function M(T){let S=n.get(T);if(T.depthTexture&&(T.depthTexture.dispose(),n.remove(T.depthTexture)),T.isWebGLCubeRenderTarget)for(let j=0;j<6;j++){if(Array.isArray(S.__webglFramebuffer[j]))for(let at=0;at<S.__webglFramebuffer[j].length;at++)i.deleteFramebuffer(S.__webglFramebuffer[j][at]);else i.deleteFramebuffer(S.__webglFramebuffer[j]);S.__webglDepthbuffer&&i.deleteRenderbuffer(S.__webglDepthbuffer[j])}else{if(Array.isArray(S.__webglFramebuffer))for(let j=0;j<S.__webglFramebuffer.length;j++)i.deleteFramebuffer(S.__webglFramebuffer[j]);else i.deleteFramebuffer(S.__webglFramebuffer);if(S.__webglDepthbuffer&&i.deleteRenderbuffer(S.__webglDepthbuffer),S.__webglMultisampledFramebuffer&&i.deleteFramebuffer(S.__webglMultisampledFramebuffer),S.__webglColorRenderbuffer)for(let j=0;j<S.__webglColorRenderbuffer.length;j++)S.__webglColorRenderbuffer[j]&&i.deleteRenderbuffer(S.__webglColorRenderbuffer[j]);S.__webglDepthRenderbuffer&&i.deleteRenderbuffer(S.__webglDepthRenderbuffer)}let X=T.textures;for(let j=0,at=X.length;j<at;j++){let J=n.get(X[j]);J.__webglTexture&&(i.deleteTexture(J.__webglTexture),o.memory.textures--),n.remove(X[j])}n.remove(T)}let R=0;function F(){R=0}function V(){let T=R;return T>=s.maxTextures&&console.warn("THREE.WebGLTextures: Trying to use "+T+" texture units while this GPU supports only "+s.maxTextures),R+=1,T}function z(T){let S=[];return S.push(T.wrapS),S.push(T.wrapT),S.push(T.wrapR||0),S.push(T.magFilter),S.push(T.minFilter),S.push(T.anisotropy),S.push(T.internalFormat),S.push(T.format),S.push(T.type),S.push(T.generateMipmaps),S.push(T.premultiplyAlpha),S.push(T.flipY),S.push(T.unpackAlignment),S.push(T.colorSpace),S.join()}function k(T,S){let X=n.get(T);if(T.isVideoTexture&&Vt(T),T.isRenderTargetTexture===!1&&T.version>0&&X.__version!==T.version){let j=T.image;if(j===null)console.warn("THREE.WebGLRenderer: Texture marked for update but no image data found.");else if(j.complete===!1)console.warn("THREE.WebGLRenderer: Texture marked for update but image is incomplete");else{H(X,T,S);return}}e.bindTexture(i.TEXTURE_2D,X.__webglTexture,i.TEXTURE0+S)}function I(T,S){let X=n.get(T);if(T.version>0&&X.__version!==T.version){H(X,T,S);return}e.bindTexture(i.TEXTURE_2D_ARRAY,X.__webglTexture,i.TEXTURE0+S)}function Y(T,S){let X=n.get(T);if(T.version>0&&X.__version!==T.version){H(X,T,S);return}e.bindTexture(i.TEXTURE_3D,X.__webglTexture,i.TEXTURE0+S)}function B(T,S){let X=n.get(T);if(T.version>0&&X.__version!==T.version){st(X,T,S);return}e.bindTexture(i.TEXTURE_CUBE_MAP,X.__webglTexture,i.TEXTURE0+S)}let Q={[Un]:i.REPEAT,[Ln]:i.CLAMP_TO_EDGE,[aa]:i.MIRRORED_REPEAT},ut={[fn]:i.NEAREST,[Wu]:i.NEAREST_MIPMAP_NEAREST,[js]:i.NEAREST_MIPMAP_LINEAR,[je]:i.LINEAR,[vo]:i.LINEAR_MIPMAP_NEAREST,[qn]:i.LINEAR_MIPMAP_LINEAR},Et={[$u]:i.NEVER,[td]:i.ALWAYS,[Zu]:i.LESS,[lh]:i.LEQUAL,[Ku]:i.EQUAL,[Qu]:i.GEQUAL,[Ju]:i.GREATER,[ju]:i.NOTEQUAL};function zt(T,S){if(S.type===_n&&t.has("OES_texture_float_linear")===!1&&(S.magFilter===je||S.magFilter===vo||S.magFilter===js||S.magFilter===qn||S.minFilter===je||S.minFilter===vo||S.minFilter===js||S.minFilter===qn)&&console.warn("THREE.WebGLRenderer: Unable to use linear filtering with floating point textures. OES_texture_float_linear not supported on this device."),i.texParameteri(T,i.TEXTURE_WRAP_S,Q[S.wrapS]),i.texParameteri(T,i.TEXTURE_WRAP_T,Q[S.wrapT]),(T===i.TEXTURE_3D||T===i.TEXTURE_2D_ARRAY)&&i.texParameteri(T,i.TEXTURE_WRAP_R,Q[S.wrapR]),i.texParameteri(T,i.TEXTURE_MAG_FILTER,ut[S.magFilter]),i.texParameteri(T,i.TEXTURE_MIN_FILTER,ut[S.minFilter]),S.compareFunction&&(i.texParameteri(T,i.TEXTURE_COMPARE_MODE,i.COMPARE_REF_TO_TEXTURE),i.texParameteri(T,i.TEXTURE_COMPARE_FUNC,Et[S.compareFunction])),t.has("EXT_texture_filter_anisotropic")===!0){if(S.magFilter===fn||S.minFilter!==js&&S.minFilter!==qn||S.type===_n&&t.has("OES_texture_float_linear")===!1)return;if(S.anisotropy>1||n.get(S).__currentAnisotropy){let X=t.get("EXT_texture_filter_anisotropic");i.texParameterf(T,X.TEXTURE_MAX_ANISOTROPY_EXT,Math.min(S.anisotropy,s.getMaxAnisotropy())),n.get(S).__currentAnisotropy=S.anisotropy}}}function $t(T,S){let X=!1;T.__webglInit===void 0&&(T.__webglInit=!0,S.addEventListener("dispose",w));let j=S.source,at=p.get(j);at===void 0&&(at={},p.set(j,at));let J=z(S);if(J!==T.__cacheKey){at[J]===void 0&&(at[J]={texture:i.createTexture(),usedTimes:0},o.memory.textures++,X=!0),at[J].usedTimes++;let Ft=at[T.__cacheKey];Ft!==void 0&&(at[T.__cacheKey].usedTimes--,Ft.usedTimes===0&&E(S)),T.__cacheKey=J,T.__webglTexture=at[J].texture}return X}function H(T,S,X){let j=i.TEXTURE_2D;(S.isDataArrayTexture||S.isCompressedArrayTexture)&&(j=i.TEXTURE_2D_ARRAY),S.isData3DTexture&&(j=i.TEXTURE_3D);let at=$t(T,S),J=S.source;e.bindTexture(j,T.__webglTexture,i.TEXTURE0+X);let Ft=n.get(J);if(J.version!==Ft.__version||at===!0){e.activeTexture(i.TEXTURE0+X);let St=he.getPrimaries(he.workingColorSpace),Rt=S.colorSpace===hi?null:he.getPrimaries(S.colorSpace),re=S.colorSpace===hi||St===Rt?i.NONE:i.BROWSER_DEFAULT_WEBGL;i.pixelStorei(i.UNPACK_FLIP_Y_WEBGL,S.flipY),i.pixelStorei(i.UNPACK_PREMULTIPLY_ALPHA_WEBGL,S.premultiplyAlpha),i.pixelStorei(i.UNPACK_ALIGNMENT,S.unpackAlignment),i.pixelStorei(i.UNPACK_COLORSPACE_CONVERSION_WEBGL,re);let lt=y(S.image,!1,s.maxTextureSize);lt=Lt(S,lt);let K=r.convert(S.format,S.colorSpace),Nt=r.convert(S.type),Zt=_(S.internalFormat,K,Nt,S.colorSpace,S.isVideoTexture);zt(j,S);let It,ne=S.mipmaps,Kt=S.isVideoTexture!==!0,Gt=Ft.__version===void 0||at===!0,L=J.dataReady,dt=C(S,lt);if(S.isDepthTexture)Zt=v(S.format===ls,S.type),Gt&&(Kt?e.texStorage2D(i.TEXTURE_2D,1,Zt,lt.width,lt.height):e.texImage2D(i.TEXTURE_2D,0,Zt,lt.width,lt.height,0,K,Nt,null));else if(S.isDataTexture)if(ne.length>0){Kt&&Gt&&e.texStorage2D(i.TEXTURE_2D,dt,Zt,ne[0].width,ne[0].height);for(let q=0,tt=ne.length;q<tt;q++)It=ne[q],Kt?L&&e.texSubImage2D(i.TEXTURE_2D,q,0,0,It.width,It.height,K,Nt,It.data):e.texImage2D(i.TEXTURE_2D,q,Zt,It.width,It.height,0,K,Nt,It.data);S.generateMipmaps=!1}else Kt?(Gt&&e.texStorage2D(i.TEXTURE_2D,dt,Zt,lt.width,lt.height),L&&e.texSubImage2D(i.TEXTURE_2D,0,0,0,lt.width,lt.height,K,Nt,lt.data)):e.texImage2D(i.TEXTURE_2D,0,Zt,lt.width,lt.height,0,K,Nt,lt.data);else if(S.isCompressedTexture)if(S.isCompressedArrayTexture){Kt&&Gt&&e.texStorage3D(i.TEXTURE_2D_ARRAY,dt,Zt,ne[0].width,ne[0].height,lt.depth);for(let q=0,tt=ne.length;q<tt;q++)if(It=ne[q],S.format!==dn)if(K!==null)if(Kt){if(L)if(S.layerUpdates.size>0){let wt=Vl(It.width,It.height,S.format,S.type);for(let bt of S.layerUpdates){let et=It.data.subarray(bt*wt/It.data.BYTES_PER_ELEMENT,(bt+1)*wt/It.data.BYTES_PER_ELEMENT);e.compressedTexSubImage3D(i.TEXTURE_2D_ARRAY,q,0,0,bt,It.width,It.height,1,K,et)}S.clearLayerUpdates()}else e.compressedTexSubImage3D(i.TEXTURE_2D_ARRAY,q,0,0,0,It.width,It.height,lt.depth,K,It.data)}else e.compressedTexImage3D(i.TEXTURE_2D_ARRAY,q,Zt,It.width,It.height,lt.depth,0,It.data,0,0);else console.warn("THREE.WebGLRenderer: Attempt to load unsupported compressed texture format in .uploadTexture()");else Kt?L&&e.texSubImage3D(i.TEXTURE_2D_ARRAY,q,0,0,0,It.width,It.height,lt.depth,K,Nt,It.data):e.texImage3D(i.TEXTURE_2D_ARRAY,q,Zt,It.width,It.height,lt.depth,0,K,Nt,It.data)}else{Kt&&Gt&&e.texStorage2D(i.TEXTURE_2D,dt,Zt,ne[0].width,ne[0].height);for(let q=0,tt=ne.length;q<tt;q++)It=ne[q],S.format!==dn?K!==null?Kt?L&&e.compressedTexSubImage2D(i.TEXTURE_2D,q,0,0,It.width,It.height,K,It.data):e.compressedTexImage2D(i.TEXTURE_2D,q,Zt,It.width,It.height,0,It.data):console.warn("THREE.WebGLRenderer: Attempt to load unsupported compressed texture format in .uploadTexture()"):Kt?L&&e.texSubImage2D(i.TEXTURE_2D,q,0,0,It.width,It.height,K,Nt,It.data):e.texImage2D(i.TEXTURE_2D,q,Zt,It.width,It.height,0,K,Nt,It.data)}else if(S.isDataArrayTexture)if(Kt){if(Gt&&e.texStorage3D(i.TEXTURE_2D_ARRAY,dt,Zt,lt.width,lt.height,lt.depth),L)if(S.layerUpdates.size>0){let q=Vl(lt.width,lt.height,S.format,S.type);for(let tt of S.layerUpdates){let wt=lt.data.subarray(tt*q/lt.data.BYTES_PER_ELEMENT,(tt+1)*q/lt.data.BYTES_PER_ELEMENT);e.texSubImage3D(i.TEXTURE_2D_ARRAY,0,0,0,tt,lt.width,lt.height,1,K,Nt,wt)}S.clearLayerUpdates()}else e.texSubImage3D(i.TEXTURE_2D_ARRAY,0,0,0,0,lt.width,lt.height,lt.depth,K,Nt,lt.data)}else e.texImage3D(i.TEXTURE_2D_ARRAY,0,Zt,lt.width,lt.height,lt.depth,0,K,Nt,lt.data);else if(S.isData3DTexture)Kt?(Gt&&e.texStorage3D(i.TEXTURE_3D,dt,Zt,lt.width,lt.height,lt.depth),L&&e.texSubImage3D(i.TEXTURE_3D,0,0,0,0,lt.width,lt.height,lt.depth,K,Nt,lt.data)):e.texImage3D(i.TEXTURE_3D,0,Zt,lt.width,lt.height,lt.depth,0,K,Nt,lt.data);else if(S.isFramebufferTexture){if(Gt)if(Kt)e.texStorage2D(i.TEXTURE_2D,dt,Zt,lt.width,lt.height);else{let q=lt.width,tt=lt.height;for(let wt=0;wt<dt;wt++)e.texImage2D(i.TEXTURE_2D,wt,Zt,q,tt,0,K,Nt,null),q>>=1,tt>>=1}}else if(ne.length>0){if(Kt&&Gt){let q=Ut(ne[0]);e.texStorage2D(i.TEXTURE_2D,dt,Zt,q.width,q.height)}for(let q=0,tt=ne.length;q<tt;q++)It=ne[q],Kt?L&&e.texSubImage2D(i.TEXTURE_2D,q,0,0,K,Nt,It):e.texImage2D(i.TEXTURE_2D,q,Zt,K,Nt,It);S.generateMipmaps=!1}else if(Kt){if(Gt){let q=Ut(lt);e.texStorage2D(i.TEXTURE_2D,dt,Zt,q.width,q.height)}L&&e.texSubImage2D(i.TEXTURE_2D,0,0,0,K,Nt,lt)}else e.texImage2D(i.TEXTURE_2D,0,Zt,K,Nt,lt);m(S)&&d(j),Ft.__version=J.version,S.onUpdate&&S.onUpdate(S)}T.__version=S.version}function st(T,S,X){if(S.image.length!==6)return;let j=$t(T,S),at=S.source;e.bindTexture(i.TEXTURE_CUBE_MAP,T.__webglTexture,i.TEXTURE0+X);let J=n.get(at);if(at.version!==J.__version||j===!0){e.activeTexture(i.TEXTURE0+X);let Ft=he.getPrimaries(he.workingColorSpace),St=S.colorSpace===hi?null:he.getPrimaries(S.colorSpace),Rt=S.colorSpace===hi||Ft===St?i.NONE:i.BROWSER_DEFAULT_WEBGL;i.pixelStorei(i.UNPACK_FLIP_Y_WEBGL,S.flipY),i.pixelStorei(i.UNPACK_PREMULTIPLY_ALPHA_WEBGL,S.premultiplyAlpha),i.pixelStorei(i.UNPACK_ALIGNMENT,S.unpackAlignment),i.pixelStorei(i.UNPACK_COLORSPACE_CONVERSION_WEBGL,Rt);let re=S.isCompressedTexture||S.image[0].isCompressedTexture,lt=S.image[0]&&S.image[0].isDataTexture,K=[];for(let tt=0;tt<6;tt++)!re&&!lt?K[tt]=y(S.image[tt],!0,s.maxCubemapSize):K[tt]=lt?S.image[tt].image:S.image[tt],K[tt]=Lt(S,K[tt]);let Nt=K[0],Zt=r.convert(S.format,S.colorSpace),It=r.convert(S.type),ne=_(S.internalFormat,Zt,It,S.colorSpace),Kt=S.isVideoTexture!==!0,Gt=J.__version===void 0||j===!0,L=at.dataReady,dt=C(S,Nt);zt(i.TEXTURE_CUBE_MAP,S);let q;if(re){Kt&&Gt&&e.texStorage2D(i.TEXTURE_CUBE_MAP,dt,ne,Nt.width,Nt.height);for(let tt=0;tt<6;tt++){q=K[tt].mipmaps;for(let wt=0;wt<q.length;wt++){let bt=q[wt];S.format!==dn?Zt!==null?Kt?L&&e.compressedTexSubImage2D(i.TEXTURE_CUBE_MAP_POSITIVE_X+tt,wt,0,0,bt.width,bt.height,Zt,bt.data):e.compressedTexImage2D(i.TEXTURE_CUBE_MAP_POSITIVE_X+tt,wt,ne,bt.width,bt.height,0,bt.data):console.warn("THREE.WebGLRenderer: Attempt to load unsupported compressed texture format in .setTextureCube()"):Kt?L&&e.texSubImage2D(i.TEXTURE_CUBE_MAP_POSITIVE_X+tt,wt,0,0,bt.width,bt.height,Zt,It,bt.data):e.texImage2D(i.TEXTURE_CUBE_MAP_POSITIVE_X+tt,wt,ne,bt.width,bt.height,0,Zt,It,bt.data)}}}else{if(q=S.mipmaps,Kt&&Gt){q.length>0&&dt++;let tt=Ut(K[0]);e.texStorage2D(i.TEXTURE_CUBE_MAP,dt,ne,tt.width,tt.height)}for(let tt=0;tt<6;tt++)if(lt){Kt?L&&e.texSubImage2D(i.TEXTURE_CUBE_MAP_POSITIVE_X+tt,0,0,0,K[tt].width,K[tt].height,Zt,It,K[tt].data):e.texImage2D(i.TEXTURE_CUBE_MAP_POSITIVE_X+tt,0,ne,K[tt].width,K[tt].height,0,Zt,It,K[tt].data);for(let wt=0;wt<q.length;wt++){let et=q[wt].image[tt].image;Kt?L&&e.texSubImage2D(i.TEXTURE_CUBE_MAP_POSITIVE_X+tt,wt+1,0,0,et.width,et.height,Zt,It,et.data):e.texImage2D(i.TEXTURE_CUBE_MAP_POSITIVE_X+tt,wt+1,ne,et.width,et.height,0,Zt,It,et.data)}}else{Kt?L&&e.texSubImage2D(i.TEXTURE_CUBE_MAP_POSITIVE_X+tt,0,0,0,Zt,It,K[tt]):e.texImage2D(i.TEXTURE_CUBE_MAP_POSITIVE_X+tt,0,ne,Zt,It,K[tt]);for(let wt=0;wt<q.length;wt++){let bt=q[wt];Kt?L&&e.texSubImage2D(i.TEXTURE_CUBE_MAP_POSITIVE_X+tt,wt+1,0,0,Zt,It,bt.image[tt]):e.texImage2D(i.TEXTURE_CUBE_MAP_POSITIVE_X+tt,wt+1,ne,Zt,It,bt.image[tt])}}}m(S)&&d(i.TEXTURE_CUBE_MAP),J.__version=at.version,S.onUpdate&&S.onUpdate(S)}T.__version=S.version}function pt(T,S,X,j,at,J){let Ft=r.convert(X.format,X.colorSpace),St=r.convert(X.type),Rt=_(X.internalFormat,Ft,St,X.colorSpace),re=n.get(S),lt=n.get(X);if(lt.__renderTarget=S,!re.__hasExternalTextures){let K=Math.max(1,S.width>>J),Nt=Math.max(1,S.height>>J);at===i.TEXTURE_3D||at===i.TEXTURE_2D_ARRAY?e.texImage3D(at,J,Rt,K,Nt,S.depth,0,Ft,St,null):e.texImage2D(at,J,Rt,K,Nt,0,Ft,St,null)}e.bindFramebuffer(i.FRAMEBUFFER,T),qt(S)?a.framebufferTexture2DMultisampleEXT(i.FRAMEBUFFER,j,at,lt.__webglTexture,0,Yt(S)):(at===i.TEXTURE_2D||at>=i.TEXTURE_CUBE_MAP_POSITIVE_X&&at<=i.TEXTURE_CUBE_MAP_NEGATIVE_Z)&&i.framebufferTexture2D(i.FRAMEBUFFER,j,at,lt.__webglTexture,J),e.bindFramebuffer(i.FRAMEBUFFER,null)}function ct(T,S,X){if(i.bindRenderbuffer(i.RENDERBUFFER,T),S.depthBuffer){let j=S.depthTexture,at=j&&j.isDepthTexture?j.type:null,J=v(S.stencilBuffer,at),Ft=S.stencilBuffer?i.DEPTH_STENCIL_ATTACHMENT:i.DEPTH_ATTACHMENT,St=Yt(S);qt(S)?a.renderbufferStorageMultisampleEXT(i.RENDERBUFFER,St,J,S.width,S.height):X?i.renderbufferStorageMultisample(i.RENDERBUFFER,St,J,S.width,S.height):i.renderbufferStorage(i.RENDERBUFFER,J,S.width,S.height),i.framebufferRenderbuffer(i.FRAMEBUFFER,Ft,i.RENDERBUFFER,T)}else{let j=S.textures;for(let at=0;at<j.length;at++){let J=j[at],Ft=r.convert(J.format,J.colorSpace),St=r.convert(J.type),Rt=_(J.internalFormat,Ft,St,J.colorSpace),re=Yt(S);X&&qt(S)===!1?i.renderbufferStorageMultisample(i.RENDERBUFFER,re,Rt,S.width,S.height):qt(S)?a.renderbufferStorageMultisampleEXT(i.RENDERBUFFER,re,Rt,S.width,S.height):i.renderbufferStorage(i.RENDERBUFFER,Rt,S.width,S.height)}}i.bindRenderbuffer(i.RENDERBUFFER,null)}function Pt(T,S){if(S&&S.isWebGLCubeRenderTarget)throw new Error("Depth Texture with cube render targets is not supported");if(e.bindFramebuffer(i.FRAMEBUFFER,T),!(S.depthTexture&&S.depthTexture.isDepthTexture))throw new Error("renderTarget.depthTexture must be an instance of THREE.DepthTexture");let j=n.get(S.depthTexture);j.__renderTarget=S,(!j.__webglTexture||S.depthTexture.image.width!==S.width||S.depthTexture.image.height!==S.height)&&(S.depthTexture.image.width=S.width,S.depthTexture.image.height=S.height,S.depthTexture.needsUpdate=!0),k(S.depthTexture,0);let at=j.__webglTexture,J=Yt(S);if(S.depthTexture.format===ns)qt(S)?a.framebufferTexture2DMultisampleEXT(i.FRAMEBUFFER,i.DEPTH_ATTACHMENT,i.TEXTURE_2D,at,0,J):i.framebufferTexture2D(i.FRAMEBUFFER,i.DEPTH_ATTACHMENT,i.TEXTURE_2D,at,0);else if(S.depthTexture.format===ls)qt(S)?a.framebufferTexture2DMultisampleEXT(i.FRAMEBUFFER,i.DEPTH_STENCIL_ATTACHMENT,i.TEXTURE_2D,at,0,J):i.framebufferTexture2D(i.FRAMEBUFFER,i.DEPTH_STENCIL_ATTACHMENT,i.TEXTURE_2D,at,0);else throw new Error("Unknown depthTexture format")}function At(T){let S=n.get(T),X=T.isWebGLCubeRenderTarget===!0;if(S.__boundDepthTexture!==T.depthTexture){let j=T.depthTexture;if(S.__depthDisposeCallback&&S.__depthDisposeCallback(),j){let at=()=>{delete S.__boundDepthTexture,delete S.__depthDisposeCallback,j.removeEventListener("dispose",at)};j.addEventListener("dispose",at),S.__depthDisposeCallback=at}S.__boundDepthTexture=j}if(T.depthTexture&&!S.__autoAllocateDepthBuffer){if(X)throw new Error("target.depthTexture not supported in Cube render targets");Pt(S.__webglFramebuffer,T)}else if(X){S.__webglDepthbuffer=[];for(let j=0;j<6;j++)if(e.bindFramebuffer(i.FRAMEBUFFER,S.__webglFramebuffer[j]),S.__webglDepthbuffer[j]===void 0)S.__webglDepthbuffer[j]=i.createRenderbuffer(),ct(S.__webglDepthbuffer[j],T,!1);else{let at=T.stencilBuffer?i.DEPTH_STENCIL_ATTACHMENT:i.DEPTH_ATTACHMENT,J=S.__webglDepthbuffer[j];i.bindRenderbuffer(i.RENDERBUFFER,J),i.framebufferRenderbuffer(i.FRAMEBUFFER,at,i.RENDERBUFFER,J)}}else if(e.bindFramebuffer(i.FRAMEBUFFER,S.__webglFramebuffer),S.__webglDepthbuffer===void 0)S.__webglDepthbuffer=i.createRenderbuffer(),ct(S.__webglDepthbuffer,T,!1);else{let j=T.stencilBuffer?i.DEPTH_STENCIL_ATTACHMENT:i.DEPTH_ATTACHMENT,at=S.__webglDepthbuffer;i.bindRenderbuffer(i.RENDERBUFFER,at),i.framebufferRenderbuffer(i.FRAMEBUFFER,j,i.RENDERBUFFER,at)}e.bindFramebuffer(i.FRAMEBUFFER,null)}function Ot(T,S,X){let j=n.get(T);S!==void 0&&pt(j.__webglFramebuffer,T,T.texture,i.COLOR_ATTACHMENT0,i.TEXTURE_2D,0),X!==void 0&&At(T)}function Xt(T){let S=T.texture,X=n.get(T),j=n.get(S);T.addEventListener("dispose",A);let at=T.textures,J=T.isWebGLCubeRenderTarget===!0,Ft=at.length>1;if(Ft||(j.__webglTexture===void 0&&(j.__webglTexture=i.createTexture()),j.__version=S.version,o.memory.textures++),J){X.__webglFramebuffer=[];for(let St=0;St<6;St++)if(S.mipmaps&&S.mipmaps.length>0){X.__webglFramebuffer[St]=[];for(let Rt=0;Rt<S.mipmaps.length;Rt++)X.__webglFramebuffer[St][Rt]=i.createFramebuffer()}else X.__webglFramebuffer[St]=i.createFramebuffer()}else{if(S.mipmaps&&S.mipmaps.length>0){X.__webglFramebuffer=[];for(let St=0;St<S.mipmaps.length;St++)X.__webglFramebuffer[St]=i.createFramebuffer()}else X.__webglFramebuffer=i.createFramebuffer();if(Ft)for(let St=0,Rt=at.length;St<Rt;St++){let re=n.get(at[St]);re.__webglTexture===void 0&&(re.__webglTexture=i.createTexture(),o.memory.textures++)}if(T.samples>0&&qt(T)===!1){X.__webglMultisampledFramebuffer=i.createFramebuffer(),X.__webglColorRenderbuffer=[],e.bindFramebuffer(i.FRAMEBUFFER,X.__webglMultisampledFramebuffer);for(let St=0;St<at.length;St++){let Rt=at[St];X.__webglColorRenderbuffer[St]=i.createRenderbuffer(),i.bindRenderbuffer(i.RENDERBUFFER,X.__webglColorRenderbuffer[St]);let re=r.convert(Rt.format,Rt.colorSpace),lt=r.convert(Rt.type),K=_(Rt.internalFormat,re,lt,Rt.colorSpace,T.isXRRenderTarget===!0),Nt=Yt(T);i.renderbufferStorageMultisample(i.RENDERBUFFER,Nt,K,T.width,T.height),i.framebufferRenderbuffer(i.FRAMEBUFFER,i.COLOR_ATTACHMENT0+St,i.RENDERBUFFER,X.__webglColorRenderbuffer[St])}i.bindRenderbuffer(i.RENDERBUFFER,null),T.depthBuffer&&(X.__webglDepthRenderbuffer=i.createRenderbuffer(),ct(X.__webglDepthRenderbuffer,T,!0)),e.bindFramebuffer(i.FRAMEBUFFER,null)}}if(J){e.bindTexture(i.TEXTURE_CUBE_MAP,j.__webglTexture),zt(i.TEXTURE_CUBE_MAP,S);for(let St=0;St<6;St++)if(S.mipmaps&&S.mipmaps.length>0)for(let Rt=0;Rt<S.mipmaps.length;Rt++)pt(X.__webglFramebuffer[St][Rt],T,S,i.COLOR_ATTACHMENT0,i.TEXTURE_CUBE_MAP_POSITIVE_X+St,Rt);else pt(X.__webglFramebuffer[St],T,S,i.COLOR_ATTACHMENT0,i.TEXTURE_CUBE_MAP_POSITIVE_X+St,0);m(S)&&d(i.TEXTURE_CUBE_MAP),e.unbindTexture()}else if(Ft){for(let St=0,Rt=at.length;St<Rt;St++){let re=at[St],lt=n.get(re);e.bindTexture(i.TEXTURE_2D,lt.__webglTexture),zt(i.TEXTURE_2D,re),pt(X.__webglFramebuffer,T,re,i.COLOR_ATTACHMENT0+St,i.TEXTURE_2D,0),m(re)&&d(i.TEXTURE_2D)}e.unbindTexture()}else{let St=i.TEXTURE_2D;if((T.isWebGL3DRenderTarget||T.isWebGLArrayRenderTarget)&&(St=T.isWebGL3DRenderTarget?i.TEXTURE_3D:i.TEXTURE_2D_ARRAY),e.bindTexture(St,j.__webglTexture),zt(St,S),S.mipmaps&&S.mipmaps.length>0)for(let Rt=0;Rt<S.mipmaps.length;Rt++)pt(X.__webglFramebuffer[Rt],T,S,i.COLOR_ATTACHMENT0,St,Rt);else pt(X.__webglFramebuffer,T,S,i.COLOR_ATTACHMENT0,St,0);m(S)&&d(St),e.unbindTexture()}T.depthBuffer&&At(T)}function Jt(T){let S=T.textures;for(let X=0,j=S.length;X<j;X++){let at=S[X];if(m(at)){let J=x(T),Ft=n.get(at).__webglTexture;e.bindTexture(J,Ft),d(J),e.unbindTexture()}}}let le=[],N=[];function de(T){if(T.samples>0){if(qt(T)===!1){let S=T.textures,X=T.width,j=T.height,at=i.COLOR_BUFFER_BIT,J=T.stencilBuffer?i.DEPTH_STENCIL_ATTACHMENT:i.DEPTH_ATTACHMENT,Ft=n.get(T),St=S.length>1;if(St)for(let Rt=0;Rt<S.length;Rt++)e.bindFramebuffer(i.FRAMEBUFFER,Ft.__webglMultisampledFramebuffer),i.framebufferRenderbuffer(i.FRAMEBUFFER,i.COLOR_ATTACHMENT0+Rt,i.RENDERBUFFER,null),e.bindFramebuffer(i.FRAMEBUFFER,Ft.__webglFramebuffer),i.framebufferTexture2D(i.DRAW_FRAMEBUFFER,i.COLOR_ATTACHMENT0+Rt,i.TEXTURE_2D,null,0);e.bindFramebuffer(i.READ_FRAMEBUFFER,Ft.__webglMultisampledFramebuffer),e.bindFramebuffer(i.DRAW_FRAMEBUFFER,Ft.__webglFramebuffer);for(let Rt=0;Rt<S.length;Rt++){if(T.resolveDepthBuffer&&(T.depthBuffer&&(at|=i.DEPTH_BUFFER_BIT),T.stencilBuffer&&T.resolveStencilBuffer&&(at|=i.STENCIL_BUFFER_BIT)),St){i.framebufferRenderbuffer(i.READ_FRAMEBUFFER,i.COLOR_ATTACHMENT0,i.RENDERBUFFER,Ft.__webglColorRenderbuffer[Rt]);let re=n.get(S[Rt]).__webglTexture;i.framebufferTexture2D(i.DRAW_FRAMEBUFFER,i.COLOR_ATTACHMENT0,i.TEXTURE_2D,re,0)}i.blitFramebuffer(0,0,X,j,0,0,X,j,at,i.NEAREST),c===!0&&(le.length=0,N.length=0,le.push(i.COLOR_ATTACHMENT0+Rt),T.depthBuffer&&T.resolveDepthBuffer===!1&&(le.push(J),N.push(J),i.invalidateFramebuffer(i.DRAW_FRAMEBUFFER,N)),i.invalidateFramebuffer(i.READ_FRAMEBUFFER,le))}if(e.bindFramebuffer(i.READ_FRAMEBUFFER,null),e.bindFramebuffer(i.DRAW_FRAMEBUFFER,null),St)for(let Rt=0;Rt<S.length;Rt++){e.bindFramebuffer(i.FRAMEBUFFER,Ft.__webglMultisampledFramebuffer),i.framebufferRenderbuffer(i.FRAMEBUFFER,i.COLOR_ATTACHMENT0+Rt,i.RENDERBUFFER,Ft.__webglColorRenderbuffer[Rt]);let re=n.get(S[Rt]).__webglTexture;e.bindFramebuffer(i.FRAMEBUFFER,Ft.__webglFramebuffer),i.framebufferTexture2D(i.DRAW_FRAMEBUFFER,i.COLOR_ATTACHMENT0+Rt,i.TEXTURE_2D,re,0)}e.bindFramebuffer(i.DRAW_FRAMEBUFFER,Ft.__webglMultisampledFramebuffer)}else if(T.depthBuffer&&T.resolveDepthBuffer===!1&&c){let S=T.stencilBuffer?i.DEPTH_STENCIL_ATTACHMENT:i.DEPTH_ATTACHMENT;i.invalidateFramebuffer(i.DRAW_FRAMEBUFFER,[S])}}}function Yt(T){return Math.min(s.maxSamples,T.samples)}function qt(T){let S=n.get(T);return T.samples>0&&t.has("WEBGL_multisampled_render_to_texture")===!0&&S.__useRenderToTexture!==!1}function Vt(T){let S=o.render.frame;h.get(T)!==S&&(h.set(T,S),T.update())}function Lt(T,S){let X=T.colorSpace,j=T.format,at=T.type;return T.isCompressedTexture===!0||T.isVideoTexture===!0||X!==_s&&X!==hi&&(he.getTransfer(X)===xe?(j!==dn||at!==Jn)&&console.warn("THREE.WebGLTextures: sRGB encoded textures have to use RGBAFormat and UnsignedByteType."):console.error("THREE.WebGLTextures: Unsupported texture color space:",X)),S}function Ut(T){return typeof HTMLImageElement<"u"&&T instanceof HTMLImageElement?(l.width=T.naturalWidth||T.width,l.height=T.naturalHeight||T.height):typeof VideoFrame<"u"&&T instanceof VideoFrame?(l.width=T.displayWidth,l.height=T.displayHeight):(l.width=T.width,l.height=T.height),l}this.allocateTextureUnit=V,this.resetTextureUnits=F,this.setTexture2D=k,this.setTexture2DArray=I,this.setTexture3D=Y,this.setTextureCube=B,this.rebindTextures=Ot,this.setupRenderTarget=Xt,this.updateRenderTargetMipmap=Jt,this.updateMultisampleRenderTarget=de,this.setupDepthRenderbuffer=At,this.setupFrameBufferTexture=pt,this.useMultisampledRTT=qt}function a0(i,t){function e(n,s=hi){let r,o=he.getTransfer(s);if(n===Jn)return i.UNSIGNED_BYTE;if(n===gc)return i.UNSIGNED_SHORT_4_4_4_4;if(n===_c)return i.UNSIGNED_SHORT_5_5_5_1;if(n===nh)return i.UNSIGNED_INT_5_9_9_9_REV;if(n===th)return i.BYTE;if(n===eh)return i.SHORT;if(n===Bs)return i.UNSIGNED_SHORT;if(n===mc)return i.INT;if(n===Ci)return i.UNSIGNED_INT;if(n===_n)return i.FLOAT;if(n===Ys)return i.HALF_FLOAT;if(n===ih)return i.ALPHA;if(n===sh)return i.RGB;if(n===dn)return i.RGBA;if(n===rh)return i.LUMINANCE;if(n===oh)return i.LUMINANCE_ALPHA;if(n===ns)return i.DEPTH_COMPONENT;if(n===ls)return i.DEPTH_STENCIL;if(n===qs)return i.RED;if(n===xc)return i.RED_INTEGER;if(n===ah)return i.RG;if(n===yc)return i.RG_INTEGER;if(n===vc)return i.RGBA_INTEGER;if(n===wr||n===Ar||n===Cr||n===Rr)if(o===xe)if(r=t.get("WEBGL_compressed_texture_s3tc_srgb"),r!==null){if(n===wr)return r.COMPRESSED_SRGB_S3TC_DXT1_EXT;if(n===Ar)return r.COMPRESSED_SRGB_ALPHA_S3TC_DXT1_EXT;if(n===Cr)return r.COMPRESSED_SRGB_ALPHA_S3TC_DXT3_EXT;if(n===Rr)return r.COMPRESSED_SRGB_ALPHA_S3TC_DXT5_EXT}else return null;else if(r=t.get("WEBGL_compressed_texture_s3tc"),r!==null){if(n===wr)return r.COMPRESSED_RGB_S3TC_DXT1_EXT;if(n===Ar)return r.COMPRESSED_RGBA_S3TC_DXT1_EXT;if(n===Cr)return r.COMPRESSED_RGBA_S3TC_DXT3_EXT;if(n===Rr)return r.COMPRESSED_RGBA_S3TC_DXT5_EXT}else return null;if(n===ca||n===la||n===ha||n===ua)if(r=t.get("WEBGL_compressed_texture_pvrtc"),r!==null){if(n===ca)return r.COMPRESSED_RGB_PVRTC_4BPPV1_IMG;if(n===la)return r.COMPRESSED_RGB_PVRTC_2BPPV1_IMG;if(n===ha)return r.COMPRESSED_RGBA_PVRTC_4BPPV1_IMG;if(n===ua)return r.COMPRESSED_RGBA_PVRTC_2BPPV1_IMG}else return null;if(n===da||n===fa||n===pa)if(r=t.get("WEBGL_compressed_texture_etc"),r!==null){if(n===da||n===fa)return o===xe?r.COMPRESSED_SRGB8_ETC2:r.COMPRESSED_RGB8_ETC2;if(n===pa)return o===xe?r.COMPRESSED_SRGB8_ALPHA8_ETC2_EAC:r.COMPRESSED_RGBA8_ETC2_EAC}else return null;if(n===ma||n===ga||n===_a||n===xa||n===ya||n===va||n===Ma||n===Sa||n===ba||n===Ea||n===Ta||n===wa||n===Aa||n===Ca)if(r=t.get("WEBGL_compressed_texture_astc"),r!==null){if(n===ma)return o===xe?r.COMPRESSED_SRGB8_ALPHA8_ASTC_4x4_KHR:r.COMPRESSED_RGBA_ASTC_4x4_KHR;if(n===ga)return o===xe?r.COMPRESSED_SRGB8_ALPHA8_ASTC_5x4_KHR:r.COMPRESSED_RGBA_ASTC_5x4_KHR;if(n===_a)return o===xe?r.COMPRESSED_SRGB8_ALPHA8_ASTC_5x5_KHR:r.COMPRESSED_RGBA_ASTC_5x5_KHR;if(n===xa)return o===xe?r.COMPRESSED_SRGB8_ALPHA8_ASTC_6x5_KHR:r.COMPRESSED_RGBA_ASTC_6x5_KHR;if(n===ya)return o===xe?r.COMPRESSED_SRGB8_ALPHA8_ASTC_6x6_KHR:r.COMPRESSED_RGBA_ASTC_6x6_KHR;if(n===va)return o===xe?r.COMPRESSED_SRGB8_ALPHA8_ASTC_8x5_KHR:r.COMPRESSED_RGBA_ASTC_8x5_KHR;if(n===Ma)return o===xe?r.COMPRESSED_SRGB8_ALPHA8_ASTC_8x6_KHR:r.COMPRESSED_RGBA_ASTC_8x6_KHR;if(n===Sa)return o===xe?r.COMPRESSED_SRGB8_ALPHA8_ASTC_8x8_KHR:r.COMPRESSED_RGBA_ASTC_8x8_KHR;if(n===ba)return o===xe?r.COMPRESSED_SRGB8_ALPHA8_ASTC_10x5_KHR:r.COMPRESSED_RGBA_ASTC_10x5_KHR;if(n===Ea)return o===xe?r.COMPRESSED_SRGB8_ALPHA8_ASTC_10x6_KHR:r.COMPRESSED_RGBA_ASTC_10x6_KHR;if(n===Ta)return o===xe?r.COMPRESSED_SRGB8_ALPHA8_ASTC_10x8_KHR:r.COMPRESSED_RGBA_ASTC_10x8_KHR;if(n===wa)return o===xe?r.COMPRESSED_SRGB8_ALPHA8_ASTC_10x10_KHR:r.COMPRESSED_RGBA_ASTC_10x10_KHR;if(n===Aa)return o===xe?r.COMPRESSED_SRGB8_ALPHA8_ASTC_12x10_KHR:r.COMPRESSED_RGBA_ASTC_12x10_KHR;if(n===Ca)return o===xe?r.COMPRESSED_SRGB8_ALPHA8_ASTC_12x12_KHR:r.COMPRESSED_RGBA_ASTC_12x12_KHR}else return null;if(n===Pr||n===Ra||n===Pa)if(r=t.get("EXT_texture_compression_bptc"),r!==null){if(n===Pr)return o===xe?r.COMPRESSED_SRGB_ALPHA_BPTC_UNORM_EXT:r.COMPRESSED_RGBA_BPTC_UNORM_EXT;if(n===Ra)return r.COMPRESSED_RGB_BPTC_SIGNED_FLOAT_EXT;if(n===Pa)return r.COMPRESSED_RGB_BPTC_UNSIGNED_FLOAT_EXT}else return null;if(n===ch||n===Ia||n===La||n===Da)if(r=t.get("EXT_texture_compression_rgtc"),r!==null){if(n===Pr)return r.COMPRESSED_RED_RGTC1_EXT;if(n===Ia)return r.COMPRESSED_SIGNED_RED_RGTC1_EXT;if(n===La)return r.COMPRESSED_RED_GREEN_RGTC2_EXT;if(n===Da)return r.COMPRESSED_SIGNED_RED_GREEN_RGTC2_EXT}else return null;return n===cs?i.UNSIGNED_INT_24_8:i[n]!==void 0?i[n]:null}return{convert:e}}var Za=class extends Ve{constructor(t=[]){super(),this.isArrayCamera=!0,this.cameras=t}},ze=class extends Re{constructor(){super(),this.isGroup=!0,this.type="Group"}},c0={type:"move"},Os=class{constructor(){this._targetRay=null,this._grip=null,this._hand=null}getHandSpace(){return this._hand===null&&(this._hand=new ze,this._hand.matrixAutoUpdate=!1,this._hand.visible=!1,this._hand.joints={},this._hand.inputState={pinching:!1}),this._hand}getTargetRaySpace(){return this._targetRay===null&&(this._targetRay=new ze,this._targetRay.matrixAutoUpdate=!1,this._targetRay.visible=!1,this._targetRay.hasLinearVelocity=!1,this._targetRay.linearVelocity=new U,this._targetRay.hasAngularVelocity=!1,this._targetRay.angularVelocity=new U),this._targetRay}getGripSpace(){return this._grip===null&&(this._grip=new ze,this._grip.matrixAutoUpdate=!1,this._grip.visible=!1,this._grip.hasLinearVelocity=!1,this._grip.linearVelocity=new U,this._grip.hasAngularVelocity=!1,this._grip.angularVelocity=new U),this._grip}dispatchEvent(t){return this._targetRay!==null&&this._targetRay.dispatchEvent(t),this._grip!==null&&this._grip.dispatchEvent(t),this._hand!==null&&this._hand.dispatchEvent(t),this}connect(t){if(t&&t.hand){let e=this._hand;if(e)for(let n of t.hand.values())this._getHandJoint(e,n)}return this.dispatchEvent({type:"connected",data:t}),this}disconnect(t){return this.dispatchEvent({type:"disconnected",data:t}),this._targetRay!==null&&(this._targetRay.visible=!1),this._grip!==null&&(this._grip.visible=!1),this._hand!==null&&(this._hand.visible=!1),this}update(t,e,n){let s=null,r=null,o=null,a=this._targetRay,c=this._grip,l=this._hand;if(t&&e.session.visibilityState!=="visible-blurred"){if(l&&t.hand){o=!0;for(let y of t.hand.values()){let m=e.getJointPose(y,n),d=this._getHandJoint(l,y);m!==null&&(d.matrix.fromArray(m.transform.matrix),d.matrix.decompose(d.position,d.rotation,d.scale),d.matrixWorldNeedsUpdate=!0,d.jointRadius=m.radius),d.visible=m!==null}let h=l.joints["index-finger-tip"],u=l.joints["thumb-tip"],p=h.position.distanceTo(u.position),f=.02,g=.005;l.inputState.pinching&&p>f+g?(l.inputState.pinching=!1,this.dispatchEvent({type:"pinchend",handedness:t.handedness,target:this})):!l.inputState.pinching&&p<=f-g&&(l.inputState.pinching=!0,this.dispatchEvent({type:"pinchstart",handedness:t.handedness,target:this}))}else c!==null&&t.gripSpace&&(r=e.getPose(t.gripSpace,n),r!==null&&(c.matrix.fromArray(r.transform.matrix),c.matrix.decompose(c.position,c.rotation,c.scale),c.matrixWorldNeedsUpdate=!0,r.linearVelocity?(c.hasLinearVelocity=!0,c.linearVelocity.copy(r.linearVelocity)):c.hasLinearVelocity=!1,r.angularVelocity?(c.hasAngularVelocity=!0,c.angularVelocity.copy(r.angularVelocity)):c.hasAngularVelocity=!1));a!==null&&(s=e.getPose(t.targetRaySpace,n),s===null&&r!==null&&(s=r),s!==null&&(a.matrix.fromArray(s.transform.matrix),a.matrix.decompose(a.position,a.rotation,a.scale),a.matrixWorldNeedsUpdate=!0,s.linearVelocity?(a.hasLinearVelocity=!0,a.linearVelocity.copy(s.linearVelocity)):a.hasLinearVelocity=!1,s.angularVelocity?(a.hasAngularVelocity=!0,a.angularVelocity.copy(s.angularVelocity)):a.hasAngularVelocity=!1,this.dispatchEvent(c0)))}return a!==null&&(a.visible=s!==null),c!==null&&(c.visible=r!==null),l!==null&&(l.visible=o!==null),this}_getHandJoint(t,e){if(t.joints[e.jointName]===void 0){let n=new ze;n.matrixAutoUpdate=!1,n.visible=!1,t.joints[e.jointName]=n,t.add(n)}return t.joints[e.jointName]}},l0=`
void main() {

	gl_Position = vec4( position, 1.0 );

}`,h0=`
uniform sampler2DArray depthColor;
uniform float depthWidth;
uniform float depthHeight;

void main() {

	vec2 coord = vec2( gl_FragCoord.x / depthWidth, gl_FragCoord.y / depthHeight );

	if ( coord.x >= 1.0 ) {

		gl_FragDepth = texture( depthColor, vec3( coord.x - 1.0, coord.y, 1 ) ).r;

	} else {

		gl_FragDepth = texture( depthColor, vec3( coord.x, coord.y, 0 ) ).r;

	}

}`,Ka=class{constructor(){this.texture=null,this.mesh=null,this.depthNear=0,this.depthFar=0}init(t,e,n){if(this.texture===null){let s=new nn,r=t.properties.get(s);r.__webglTexture=e.texture,(e.depthNear!=n.depthNear||e.depthFar!=n.depthFar)&&(this.depthNear=e.depthNear,this.depthFar=e.depthFar),this.texture=s}}getMesh(t){if(this.texture!==null&&this.mesh===null){let e=t.cameras[0].viewport,n=new Pe({vertexShader:l0,fragmentShader:h0,uniforms:{depthColor:{value:this.texture},depthWidth:{value:e.z},depthHeight:{value:e.w}}});this.mesh=new se(new wn(20,20),n)}return this.mesh}reset(){this.texture=null,this.mesh=null}getDepthTexture(){return this.texture}},Ja=class extends jn{constructor(t,e){super();let n=this,s=null,r=1,o=null,a="local-floor",c=1,l=null,h=null,u=null,p=null,f=null,g=null,y=new Ka,m=e.getContextAttributes(),d=null,x=null,_=[],v=[],C=new Wt,w=null,A=new Ve;A.viewport=new be;let P=new Ve;P.viewport=new be;let E=[A,P],M=new Za,R=null,F=null;this.cameraAutoUpdate=!0,this.enabled=!1,this.isPresenting=!1,this.getController=function(H){let st=_[H];return st===void 0&&(st=new Os,_[H]=st),st.getTargetRaySpace()},this.getControllerGrip=function(H){let st=_[H];return st===void 0&&(st=new Os,_[H]=st),st.getGripSpace()},this.getHand=function(H){let st=_[H];return st===void 0&&(st=new Os,_[H]=st),st.getHandSpace()};function V(H){let st=v.indexOf(H.inputSource);if(st===-1)return;let pt=_[st];pt!==void 0&&(pt.update(H.inputSource,H.frame,l||o),pt.dispatchEvent({type:H.type,data:H.inputSource}))}function z(){s.removeEventListener("select",V),s.removeEventListener("selectstart",V),s.removeEventListener("selectend",V),s.removeEventListener("squeeze",V),s.removeEventListener("squeezestart",V),s.removeEventListener("squeezeend",V),s.removeEventListener("end",z),s.removeEventListener("inputsourceschange",k);for(let H=0;H<_.length;H++){let st=v[H];st!==null&&(v[H]=null,_[H].disconnect(st))}R=null,F=null,y.reset(),t.setRenderTarget(d),f=null,p=null,u=null,s=null,x=null,$t.stop(),n.isPresenting=!1,t.setPixelRatio(w),t.setSize(C.width,C.height,!1),n.dispatchEvent({type:"sessionend"})}this.setFramebufferScaleFactor=function(H){r=H,n.isPresenting===!0&&console.warn("THREE.WebXRManager: Cannot change framebuffer scale while presenting.")},this.setReferenceSpaceType=function(H){a=H,n.isPresenting===!0&&console.warn("THREE.WebXRManager: Cannot change reference space type while presenting.")},this.getReferenceSpace=function(){return l||o},this.setReferenceSpace=function(H){l=H},this.getBaseLayer=function(){return p!==null?p:f},this.getBinding=function(){return u},this.getFrame=function(){return g},this.getSession=function(){return s},this.setSession=async function(H){if(s=H,s!==null){if(d=t.getRenderTarget(),s.addEventListener("select",V),s.addEventListener("selectstart",V),s.addEventListener("selectend",V),s.addEventListener("squeeze",V),s.addEventListener("squeezestart",V),s.addEventListener("squeezeend",V),s.addEventListener("end",z),s.addEventListener("inputsourceschange",k),m.xrCompatible!==!0&&await e.makeXRCompatible(),w=t.getPixelRatio(),t.getSize(C),s.renderState.layers===void 0){let st={antialias:m.antialias,alpha:!0,depth:m.depth,stencil:m.stencil,framebufferScaleFactor:r};f=new XRWebGLLayer(s,e,st),s.updateRenderState({baseLayer:f}),t.setPixelRatio(1),t.setSize(f.framebufferWidth,f.framebufferHeight,!1),x=new En(f.framebufferWidth,f.framebufferHeight,{format:dn,type:Jn,colorSpace:t.outputColorSpace,stencilBuffer:m.stencil})}else{let st=null,pt=null,ct=null;m.depth&&(ct=m.stencil?e.DEPTH24_STENCIL8:e.DEPTH_COMPONENT24,st=m.stencil?ls:ns,pt=m.stencil?cs:Ci);let Pt={colorFormat:e.RGBA8,depthFormat:ct,scaleFactor:r};u=new XRWebGLBinding(s,e),p=u.createProjectionLayer(Pt),s.updateRenderState({layers:[p]}),t.setPixelRatio(1),t.setSize(p.textureWidth,p.textureHeight,!1),x=new En(p.textureWidth,p.textureHeight,{format:dn,type:Jn,depthTexture:new Vr(p.textureWidth,p.textureHeight,pt,void 0,void 0,void 0,void 0,void 0,void 0,st),stencilBuffer:m.stencil,colorSpace:t.outputColorSpace,samples:m.antialias?4:0,resolveDepthBuffer:p.ignoreDepthValues===!1})}x.isXRRenderTarget=!0,this.setFoveation(c),l=null,o=await s.requestReferenceSpace(a),$t.setContext(s),$t.start(),n.isPresenting=!0,n.dispatchEvent({type:"sessionstart"})}},this.getEnvironmentBlendMode=function(){if(s!==null)return s.environmentBlendMode},this.getDepthTexture=function(){return y.getDepthTexture()};function k(H){for(let st=0;st<H.removed.length;st++){let pt=H.removed[st],ct=v.indexOf(pt);ct>=0&&(v[ct]=null,_[ct].disconnect(pt))}for(let st=0;st<H.added.length;st++){let pt=H.added[st],ct=v.indexOf(pt);if(ct===-1){for(let At=0;At<_.length;At++)if(At>=v.length){v.push(pt),ct=At;break}else if(v[At]===null){v[At]=pt,ct=At;break}if(ct===-1)break}let Pt=_[ct];Pt&&Pt.connect(pt)}}let I=new U,Y=new U;function B(H,st,pt){I.setFromMatrixPosition(st.matrixWorld),Y.setFromMatrixPosition(pt.matrixWorld);let ct=I.distanceTo(Y),Pt=st.projectionMatrix.elements,At=pt.projectionMatrix.elements,Ot=Pt[14]/(Pt[10]-1),Xt=Pt[14]/(Pt[10]+1),Jt=(Pt[9]+1)/Pt[5],le=(Pt[9]-1)/Pt[5],N=(Pt[8]-1)/Pt[0],de=(At[8]+1)/At[0],Yt=Ot*N,qt=Ot*de,Vt=ct/(-N+de),Lt=Vt*-N;if(st.matrixWorld.decompose(H.position,H.quaternion,H.scale),H.translateX(Lt),H.translateZ(Vt),H.matrixWorld.compose(H.position,H.quaternion,H.scale),H.matrixWorldInverse.copy(H.matrixWorld).invert(),Pt[10]===-1)H.projectionMatrix.copy(st.projectionMatrix),H.projectionMatrixInverse.copy(st.projectionMatrixInverse);else{let Ut=Ot+Vt,T=Xt+Vt,S=Yt-Lt,X=qt+(ct-Lt),j=Jt*Xt/T*Ut,at=le*Xt/T*Ut;H.projectionMatrix.makePerspective(S,X,j,at,Ut,T),H.projectionMatrixInverse.copy(H.projectionMatrix).invert()}}function Q(H,st){st===null?H.matrixWorld.copy(H.matrix):H.matrixWorld.multiplyMatrices(st.matrixWorld,H.matrix),H.matrixWorldInverse.copy(H.matrixWorld).invert()}this.updateCamera=function(H){if(s===null)return;let st=H.near,pt=H.far;y.texture!==null&&(y.depthNear>0&&(st=y.depthNear),y.depthFar>0&&(pt=y.depthFar)),M.near=P.near=A.near=st,M.far=P.far=A.far=pt,(R!==M.near||F!==M.far)&&(s.updateRenderState({depthNear:M.near,depthFar:M.far}),R=M.near,F=M.far),A.layers.mask=H.layers.mask|2,P.layers.mask=H.layers.mask|4,M.layers.mask=A.layers.mask|P.layers.mask;let ct=H.parent,Pt=M.cameras;Q(M,ct);for(let At=0;At<Pt.length;At++)Q(Pt[At],ct);Pt.length===2?B(M,A,P):M.projectionMatrix.copy(A.projectionMatrix),ut(H,M,ct)};function ut(H,st,pt){pt===null?H.matrix.copy(st.matrixWorld):(H.matrix.copy(pt.matrixWorld),H.matrix.invert(),H.matrix.multiply(st.matrixWorld)),H.matrix.decompose(H.position,H.quaternion,H.scale),H.updateMatrixWorld(!0),H.projectionMatrix.copy(st.projectionMatrix),H.projectionMatrixInverse.copy(st.projectionMatrixInverse),H.isPerspectiveCamera&&(H.fov=zs*2*Math.atan(1/H.projectionMatrix.elements[5]),H.zoom=1)}this.getCamera=function(){return M},this.getFoveation=function(){if(!(p===null&&f===null))return c},this.setFoveation=function(H){c=H,p!==null&&(p.fixedFoveation=H),f!==null&&f.fixedFoveation!==void 0&&(f.fixedFoveation=H)},this.hasDepthSensing=function(){return y.texture!==null},this.getDepthSensingMesh=function(){return y.getMesh(M)};let Et=null;function zt(H,st){if(h=st.getViewerPose(l||o),g=st,h!==null){let pt=h.views;f!==null&&(t.setRenderTargetFramebuffer(x,f.framebuffer),t.setRenderTarget(x));let ct=!1;pt.length!==M.cameras.length&&(M.cameras.length=0,ct=!0);for(let At=0;At<pt.length;At++){let Ot=pt[At],Xt=null;if(f!==null)Xt=f.getViewport(Ot);else{let le=u.getViewSubImage(p,Ot);Xt=le.viewport,At===0&&(t.setRenderTargetTextures(x,le.colorTexture,p.ignoreDepthValues?void 0:le.depthStencilTexture),t.setRenderTarget(x))}let Jt=E[At];Jt===void 0&&(Jt=new Ve,Jt.layers.enable(At),Jt.viewport=new be,E[At]=Jt),Jt.matrix.fromArray(Ot.transform.matrix),Jt.matrix.decompose(Jt.position,Jt.quaternion,Jt.scale),Jt.projectionMatrix.fromArray(Ot.projectionMatrix),Jt.projectionMatrixInverse.copy(Jt.projectionMatrix).invert(),Jt.viewport.set(Xt.x,Xt.y,Xt.width,Xt.height),At===0&&(M.matrix.copy(Jt.matrix),M.matrix.decompose(M.position,M.quaternion,M.scale)),ct===!0&&M.cameras.push(Jt)}let Pt=s.enabledFeatures;if(Pt&&Pt.includes("depth-sensing")){let At=u.getDepthInformation(pt[0]);At&&At.isValid&&At.texture&&y.init(t,At,s.renderState)}}for(let pt=0;pt<_.length;pt++){let ct=v[pt],Pt=_[pt];ct!==null&&Pt!==void 0&&Pt.update(ct,st,l||o)}Et&&Et(H,st),st.detectedPlanes&&n.dispatchEvent({type:"planesdetected",data:st}),g=null}let $t=new fh;$t.setAnimationLoop(zt),this.setAnimationLoop=function(H){Et=H},this.dispose=function(){}}},Ei=new sn,u0=new ce;function d0(i,t){function e(m,d){m.matrixAutoUpdate===!0&&m.updateMatrix(),d.value.copy(m.matrix)}function n(m,d){d.color.getRGB(m.fogColor.value,dh(i)),d.isFog?(m.fogNear.value=d.near,m.fogFar.value=d.far):d.isFogExp2&&(m.fogDensity.value=d.density)}function s(m,d,x,_,v){d.isMeshBasicMaterial||d.isMeshLambertMaterial?r(m,d):d.isMeshToonMaterial?(r(m,d),u(m,d)):d.isMeshPhongMaterial?(r(m,d),h(m,d)):d.isMeshStandardMaterial?(r(m,d),p(m,d),d.isMeshPhysicalMaterial&&f(m,d,v)):d.isMeshMatcapMaterial?(r(m,d),g(m,d)):d.isMeshDepthMaterial?r(m,d):d.isMeshDistanceMaterial?(r(m,d),y(m,d)):d.isMeshNormalMaterial?r(m,d):d.isLineBasicMaterial?(o(m,d),d.isLineDashedMaterial&&a(m,d)):d.isPointsMaterial?c(m,d,x,_):d.isSpriteMaterial?l(m,d):d.isShadowMaterial?(m.color.value.copy(d.color),m.opacity.value=d.opacity):d.isShaderMaterial&&(d.uniformsNeedUpdate=!1)}function r(m,d){m.opacity.value=d.opacity,d.color&&m.diffuse.value.copy(d.color),d.emissive&&m.emissive.value.copy(d.emissive).multiplyScalar(d.emissiveIntensity),d.map&&(m.map.value=d.map,e(d.map,m.mapTransform)),d.alphaMap&&(m.alphaMap.value=d.alphaMap,e(d.alphaMap,m.alphaMapTransform)),d.bumpMap&&(m.bumpMap.value=d.bumpMap,e(d.bumpMap,m.bumpMapTransform),m.bumpScale.value=d.bumpScale,d.side===ke&&(m.bumpScale.value*=-1)),d.normalMap&&(m.normalMap.value=d.normalMap,e(d.normalMap,m.normalMapTransform),m.normalScale.value.copy(d.normalScale),d.side===ke&&m.normalScale.value.negate()),d.displacementMap&&(m.displacementMap.value=d.displacementMap,e(d.displacementMap,m.displacementMapTransform),m.displacementScale.value=d.displacementScale,m.displacementBias.value=d.displacementBias),d.emissiveMap&&(m.emissiveMap.value=d.emissiveMap,e(d.emissiveMap,m.emissiveMapTransform)),d.specularMap&&(m.specularMap.value=d.specularMap,e(d.specularMap,m.specularMapTransform)),d.alphaTest>0&&(m.alphaTest.value=d.alphaTest);let x=t.get(d),_=x.envMap,v=x.envMapRotation;_&&(m.envMap.value=_,Ei.copy(v),Ei.x*=-1,Ei.y*=-1,Ei.z*=-1,_.isCubeTexture&&_.isRenderTargetTexture===!1&&(Ei.y*=-1,Ei.z*=-1),m.envMapRotation.value.setFromMatrix4(u0.makeRotationFromEuler(Ei)),m.flipEnvMap.value=_.isCubeTexture&&_.isRenderTargetTexture===!1?-1:1,m.reflectivity.value=d.reflectivity,m.ior.value=d.ior,m.refractionRatio.value=d.refractionRatio),d.lightMap&&(m.lightMap.value=d.lightMap,m.lightMapIntensity.value=d.lightMapIntensity,e(d.lightMap,m.lightMapTransform)),d.aoMap&&(m.aoMap.value=d.aoMap,m.aoMapIntensity.value=d.aoMapIntensity,e(d.aoMap,m.aoMapTransform))}function o(m,d){m.diffuse.value.copy(d.color),m.opacity.value=d.opacity,d.map&&(m.map.value=d.map,e(d.map,m.mapTransform))}function a(m,d){m.dashSize.value=d.dashSize,m.totalSize.value=d.dashSize+d.gapSize,m.scale.value=d.scale}function c(m,d,x,_){m.diffuse.value.copy(d.color),m.opacity.value=d.opacity,m.size.value=d.size*x,m.scale.value=_*.5,d.map&&(m.map.value=d.map,e(d.map,m.uvTransform)),d.alphaMap&&(m.alphaMap.value=d.alphaMap,e(d.alphaMap,m.alphaMapTransform)),d.alphaTest>0&&(m.alphaTest.value=d.alphaTest)}function l(m,d){m.diffuse.value.copy(d.color),m.opacity.value=d.opacity,m.rotation.value=d.rotation,d.map&&(m.map.value=d.map,e(d.map,m.mapTransform)),d.alphaMap&&(m.alphaMap.value=d.alphaMap,e(d.alphaMap,m.alphaMapTransform)),d.alphaTest>0&&(m.alphaTest.value=d.alphaTest)}function h(m,d){m.specular.value.copy(d.specular),m.shininess.value=Math.max(d.shininess,1e-4)}function u(m,d){d.gradientMap&&(m.gradientMap.value=d.gradientMap)}function p(m,d){m.metalness.value=d.metalness,d.metalnessMap&&(m.metalnessMap.value=d.metalnessMap,e(d.metalnessMap,m.metalnessMapTransform)),m.roughness.value=d.roughness,d.roughnessMap&&(m.roughnessMap.value=d.roughnessMap,e(d.roughnessMap,m.roughnessMapTransform)),d.envMap&&(m.envMapIntensity.value=d.envMapIntensity)}function f(m,d,x){m.ior.value=d.ior,d.sheen>0&&(m.sheenColor.value.copy(d.sheenColor).multiplyScalar(d.sheen),m.sheenRoughness.value=d.sheenRoughness,d.sheenColorMap&&(m.sheenColorMap.value=d.sheenColorMap,e(d.sheenColorMap,m.sheenColorMapTransform)),d.sheenRoughnessMap&&(m.sheenRoughnessMap.value=d.sheenRoughnessMap,e(d.sheenRoughnessMap,m.sheenRoughnessMapTransform))),d.clearcoat>0&&(m.clearcoat.value=d.clearcoat,m.clearcoatRoughness.value=d.clearcoatRoughness,d.clearcoatMap&&(m.clearcoatMap.value=d.clearcoatMap,e(d.clearcoatMap,m.clearcoatMapTransform)),d.clearcoatRoughnessMap&&(m.clearcoatRoughnessMap.value=d.clearcoatRoughnessMap,e(d.clearcoatRoughnessMap,m.clearcoatRoughnessMapTransform)),d.clearcoatNormalMap&&(m.clearcoatNormalMap.value=d.clearcoatNormalMap,e(d.clearcoatNormalMap,m.clearcoatNormalMapTransform),m.clearcoatNormalScale.value.copy(d.clearcoatNormalScale),d.side===ke&&m.clearcoatNormalScale.value.negate())),d.dispersion>0&&(m.dispersion.value=d.dispersion),d.iridescence>0&&(m.iridescence.value=d.iridescence,m.iridescenceIOR.value=d.iridescenceIOR,m.iridescenceThicknessMinimum.value=d.iridescenceThicknessRange[0],m.iridescenceThicknessMaximum.value=d.iridescenceThicknessRange[1],d.iridescenceMap&&(m.iridescenceMap.value=d.iridescenceMap,e(d.iridescenceMap,m.iridescenceMapTransform)),d.iridescenceThicknessMap&&(m.iridescenceThicknessMap.value=d.iridescenceThicknessMap,e(d.iridescenceThicknessMap,m.iridescenceThicknessMapTransform))),d.transmission>0&&(m.transmission.value=d.transmission,m.transmissionSamplerMap.value=x.texture,m.transmissionSamplerSize.value.set(x.width,x.height),d.transmissionMap&&(m.transmissionMap.value=d.transmissionMap,e(d.transmissionMap,m.transmissionMapTransform)),m.thickness.value=d.thickness,d.thicknessMap&&(m.thicknessMap.value=d.thicknessMap,e(d.thicknessMap,m.thicknessMapTransform)),m.attenuationDistance.value=d.attenuationDistance,m.attenuationColor.value.copy(d.attenuationColor)),d.anisotropy>0&&(m.anisotropyVector.value.set(d.anisotropy*Math.cos(d.anisotropyRotation),d.anisotropy*Math.sin(d.anisotropyRotation)),d.anisotropyMap&&(m.anisotropyMap.value=d.anisotropyMap,e(d.anisotropyMap,m.anisotropyMapTransform))),m.specularIntensity.value=d.specularIntensity,m.specularColor.value.copy(d.specularColor),d.specularColorMap&&(m.specularColorMap.value=d.specularColorMap,e(d.specularColorMap,m.specularColorMapTransform)),d.specularIntensityMap&&(m.specularIntensityMap.value=d.specularIntensityMap,e(d.specularIntensityMap,m.specularIntensityMapTransform))}function g(m,d){d.matcap&&(m.matcap.value=d.matcap)}function y(m,d){let x=t.get(d).light;m.referencePosition.value.setFromMatrixPosition(x.matrixWorld),m.nearDistance.value=x.shadow.camera.near,m.farDistance.value=x.shadow.camera.far}return{refreshFogUniforms:n,refreshMaterialUniforms:s}}function f0(i,t,e,n){let s={},r={},o=[],a=i.getParameter(i.MAX_UNIFORM_BUFFER_BINDINGS);function c(x,_){let v=_.program;n.uniformBlockBinding(x,v)}function l(x,_){let v=s[x.id];v===void 0&&(g(x),v=h(x),s[x.id]=v,x.addEventListener("dispose",m));let C=_.program;n.updateUBOMapping(x,C);let w=t.render.frame;r[x.id]!==w&&(p(x),r[x.id]=w)}function h(x){let _=u();x.__bindingPointIndex=_;let v=i.createBuffer(),C=x.__size,w=x.usage;return i.bindBuffer(i.UNIFORM_BUFFER,v),i.bufferData(i.UNIFORM_BUFFER,C,w),i.bindBuffer(i.UNIFORM_BUFFER,null),i.bindBufferBase(i.UNIFORM_BUFFER,_,v),v}function u(){for(let x=0;x<a;x++)if(o.indexOf(x)===-1)return o.push(x),x;return console.error("THREE.WebGLRenderer: Maximum number of simultaneously usable uniforms groups reached."),0}function p(x){let _=s[x.id],v=x.uniforms,C=x.__cache;i.bindBuffer(i.UNIFORM_BUFFER,_);for(let w=0,A=v.length;w<A;w++){let P=Array.isArray(v[w])?v[w]:[v[w]];for(let E=0,M=P.length;E<M;E++){let R=P[E];if(f(R,w,E,C)===!0){let F=R.__offset,V=Array.isArray(R.value)?R.value:[R.value],z=0;for(let k=0;k<V.length;k++){let I=V[k],Y=y(I);typeof I=="number"||typeof I=="boolean"?(R.__data[0]=I,i.bufferSubData(i.UNIFORM_BUFFER,F+z,R.__data)):I.isMatrix3?(R.__data[0]=I.elements[0],R.__data[1]=I.elements[1],R.__data[2]=I.elements[2],R.__data[3]=0,R.__data[4]=I.elements[3],R.__data[5]=I.elements[4],R.__data[6]=I.elements[5],R.__data[7]=0,R.__data[8]=I.elements[6],R.__data[9]=I.elements[7],R.__data[10]=I.elements[8],R.__data[11]=0):(I.toArray(R.__data,z),z+=Y.storage/Float32Array.BYTES_PER_ELEMENT)}i.bufferSubData(i.UNIFORM_BUFFER,F,R.__data)}}}i.bindBuffer(i.UNIFORM_BUFFER,null)}function f(x,_,v,C){let w=x.value,A=_+"_"+v;if(C[A]===void 0)return typeof w=="number"||typeof w=="boolean"?C[A]=w:C[A]=w.clone(),!0;{let P=C[A];if(typeof w=="number"||typeof w=="boolean"){if(P!==w)return C[A]=w,!0}else if(P.equals(w)===!1)return P.copy(w),!0}return!1}function g(x){let _=x.uniforms,v=0,C=16;for(let A=0,P=_.length;A<P;A++){let E=Array.isArray(_[A])?_[A]:[_[A]];for(let M=0,R=E.length;M<R;M++){let F=E[M],V=Array.isArray(F.value)?F.value:[F.value];for(let z=0,k=V.length;z<k;z++){let I=V[z],Y=y(I),B=v%C,Q=B%Y.boundary,ut=B+Q;v+=Q,ut!==0&&C-ut<Y.storage&&(v+=C-ut),F.__data=new Float32Array(Y.storage/Float32Array.BYTES_PER_ELEMENT),F.__offset=v,v+=Y.storage}}}let w=v%C;return w>0&&(v+=C-w),x.__size=v,x.__cache={},this}function y(x){let _={boundary:0,storage:0};return typeof x=="number"||typeof x=="boolean"?(_.boundary=4,_.storage=4):x.isVector2?(_.boundary=8,_.storage=8):x.isVector3||x.isColor?(_.boundary=16,_.storage=12):x.isVector4?(_.boundary=16,_.storage=16):x.isMatrix3?(_.boundary=48,_.storage=48):x.isMatrix4?(_.boundary=64,_.storage=64):x.isTexture?console.warn("THREE.WebGLRenderer: Texture samplers can not be part of an uniforms group."):console.warn("THREE.WebGLRenderer: Unsupported uniform value type.",x),_}function m(x){let _=x.target;_.removeEventListener("dispose",m);let v=o.indexOf(_.__bindingPointIndex);o.splice(v,1),i.deleteBuffer(s[_.id]),delete s[_.id],delete r[_.id]}function d(){for(let x in s)i.deleteBuffer(s[x]);o=[],s={},r={}}return{bind:c,update:l,dispose:d}}var Gr=class{constructor(t={}){let{canvas:e=_d(),context:n=null,depth:s=!0,stencil:r=!1,alpha:o=!1,antialias:a=!1,premultipliedAlpha:c=!0,preserveDrawingBuffer:l=!1,powerPreference:h="default",failIfMajorPerformanceCaveat:u=!1,reverseDepthBuffer:p=!1}=t;this.isWebGLRenderer=!0;let f;if(n!==null){if(typeof WebGLRenderingContext<"u"&&n instanceof WebGLRenderingContext)throw new Error("THREE.WebGLRenderer: WebGL 1 is not supported since r163.");f=n.getContextAttributes().alpha}else f=o;let g=new Uint32Array(4),y=new Int32Array(4),m=null,d=null,x=[],_=[];this.domElement=e,this.debug={checkShaderErrors:!0,onShaderError:null},this.autoClear=!0,this.autoClearColor=!0,this.autoClearDepth=!0,this.autoClearStencil=!0,this.sortObjects=!0,this.clippingPlanes=[],this.localClippingEnabled=!1,this._outputColorSpace=we,this.toneMapping=fi,this.toneMappingExposure=1;let v=this,C=!1,w=0,A=0,P=null,E=-1,M=null,R=new be,F=new be,V=null,z=new _t(0),k=0,I=e.width,Y=e.height,B=1,Q=null,ut=null,Et=new be(0,0,I,Y),zt=new be(0,0,I,Y),$t=!1,H=new ks,st=!1,pt=!1,ct=new ce,Pt=new ce,At=new U,Ot=new be,Xt={background:null,fog:null,environment:null,overrideMaterial:null,isScene:!0},Jt=!1;function le(){return P===null?B:1}let N=n;function de(b,D){return e.getContext(b,D)}try{let b={alpha:!0,depth:s,stencil:r,antialias:a,premultipliedAlpha:c,preserveDrawingBuffer:l,powerPreference:h,failIfMajorPerformanceCaveat:u};if("setAttribute"in e&&e.setAttribute("data-engine",`three.js r${uc}`),e.addEventListener("webglcontextlost",tt,!1),e.addEventListener("webglcontextrestored",wt,!1),e.addEventListener("webglcontextcreationerror",bt,!1),N===null){let D="webgl2";if(N=de(D,b),N===null)throw de(D)?new Error("Error creating WebGL context with your selected attributes."):new Error("Error creating WebGL context.")}}catch(b){throw console.error("THREE.WebGLRenderer: "+b.message),b}let Yt,qt,Vt,Lt,Ut,T,S,X,j,at,J,Ft,St,Rt,re,lt,K,Nt,Zt,It,ne,Kt,Gt,L;function dt(){Yt=new Rm(N),Yt.init(),Kt=new a0(N,Yt),qt=new bm(N,Yt,t,Kt),Vt=new s0(N,Yt),qt.reverseDepthBuffer&&p&&Vt.buffers.depth.setReversed(!0),Lt=new Lm(N),Ut=new Xg,T=new o0(N,Yt,Vt,Ut,qt,Kt,Lt),S=new Tm(v),X=new Cm(v),j=new Bd(N),Gt=new Mm(N,j),at=new Pm(N,j,Lt,Gt),J=new Um(N,at,j,Lt),Zt=new Dm(N,qt,T),lt=new Em(Ut),Ft=new Wg(v,S,X,Yt,qt,Gt,lt),St=new d0(v,Ut),Rt=new qg,re=new Qg(Yt),Nt=new vm(v,S,X,Vt,J,f,c),K=new n0(v,J,qt),L=new f0(N,Lt,qt,Vt),It=new Sm(N,Yt,Lt),ne=new Im(N,Yt,Lt),Lt.programs=Ft.programs,v.capabilities=qt,v.extensions=Yt,v.properties=Ut,v.renderLists=Rt,v.shadowMap=K,v.state=Vt,v.info=Lt}dt();let q=new Ja(v,N);this.xr=q,this.getContext=function(){return N},this.getContextAttributes=function(){return N.getContextAttributes()},this.forceContextLoss=function(){let b=Yt.get("WEBGL_lose_context");b&&b.loseContext()},this.forceContextRestore=function(){let b=Yt.get("WEBGL_lose_context");b&&b.restoreContext()},this.getPixelRatio=function(){return B},this.setPixelRatio=function(b){b!==void 0&&(B=b,this.setSize(I,Y,!1))},this.getSize=function(b){return b.set(I,Y)},this.setSize=function(b,D,G=!0){if(q.isPresenting){console.warn("THREE.WebGLRenderer: Can't change size while VR device is presenting.");return}I=b,Y=D,e.width=Math.floor(b*B),e.height=Math.floor(D*B),G===!0&&(e.style.width=b+"px",e.style.height=D+"px"),this.setViewport(0,0,b,D)},this.getDrawingBufferSize=function(b){return b.set(I*B,Y*B).floor()},this.setDrawingBufferSize=function(b,D,G){I=b,Y=D,B=G,e.width=Math.floor(b*G),e.height=Math.floor(D*G),this.setViewport(0,0,b,D)},this.getCurrentViewport=function(b){return b.copy(R)},this.getViewport=function(b){return b.copy(Et)},this.setViewport=function(b,D,G,W){b.isVector4?Et.set(b.x,b.y,b.z,b.w):Et.set(b,D,G,W),Vt.viewport(R.copy(Et).multiplyScalar(B).round())},this.getScissor=function(b){return b.copy(zt)},this.setScissor=function(b,D,G,W){b.isVector4?zt.set(b.x,b.y,b.z,b.w):zt.set(b,D,G,W),Vt.scissor(F.copy(zt).multiplyScalar(B).round())},this.getScissorTest=function(){return $t},this.setScissorTest=function(b){Vt.setScissorTest($t=b)},this.setOpaqueSort=function(b){Q=b},this.setTransparentSort=function(b){ut=b},this.getClearColor=function(b){return b.copy(Nt.getClearColor())},this.setClearColor=function(){Nt.setClearColor.apply(Nt,arguments)},this.getClearAlpha=function(){return Nt.getClearAlpha()},this.setClearAlpha=function(){Nt.setClearAlpha.apply(Nt,arguments)},this.clear=function(b=!0,D=!0,G=!0){let W=0;if(b){let O=!1;if(P!==null){let ht=P.texture.format;O=ht===vc||ht===yc||ht===xc}if(O){let ht=P.texture.type,vt=ht===Jn||ht===Ci||ht===Bs||ht===cs||ht===gc||ht===_c,Ct=Nt.getClearColor(),Bt=Nt.getClearAlpha(),jt=Ct.r,Qt=Ct.g,Dt=Ct.b;vt?(g[0]=jt,g[1]=Qt,g[2]=Dt,g[3]=Bt,N.clearBufferuiv(N.COLOR,0,g)):(y[0]=jt,y[1]=Qt,y[2]=Dt,y[3]=Bt,N.clearBufferiv(N.COLOR,0,y))}else W|=N.COLOR_BUFFER_BIT}D&&(W|=N.DEPTH_BUFFER_BIT),G&&(W|=N.STENCIL_BUFFER_BIT,this.state.buffers.stencil.setMask(4294967295)),N.clear(W)},this.clearColor=function(){this.clear(!0,!1,!1)},this.clearDepth=function(){this.clear(!1,!0,!1)},this.clearStencil=function(){this.clear(!1,!1,!0)},this.dispose=function(){e.removeEventListener("webglcontextlost",tt,!1),e.removeEventListener("webglcontextrestored",wt,!1),e.removeEventListener("webglcontextcreationerror",bt,!1),Rt.dispose(),re.dispose(),Ut.dispose(),S.dispose(),X.dispose(),J.dispose(),Gt.dispose(),L.dispose(),Ft.dispose(),q.dispose(),q.removeEventListener("sessionstart",te),q.removeEventListener("sessionend",He),Ze.stop()};function tt(b){b.preventDefault(),console.log("THREE.WebGLRenderer: Context Lost."),C=!0}function wt(){console.log("THREE.WebGLRenderer: Context Restored."),C=!1;let b=Lt.autoReset,D=K.enabled,G=K.autoUpdate,W=K.needsUpdate,O=K.type;dt(),Lt.autoReset=b,K.enabled=D,K.autoUpdate=G,K.needsUpdate=W,K.type=O}function bt(b){console.error("THREE.WebGLRenderer: A WebGL context could not be created. Reason: ",b.statusMessage)}function et(b){let D=b.target;D.removeEventListener("dispose",et),ot(D)}function ot(b){$(b),Ut.remove(b)}function $(b){let D=Ut.get(b).programs;D!==void 0&&(D.forEach(function(G){Ft.releaseProgram(G)}),b.isShaderMaterial&&Ft.releaseShaderCache(b))}this.renderBufferDirect=function(b,D,G,W,O,ht){D===null&&(D=Xt);let vt=O.isMesh&&O.matrixWorld.determinant()<0,Ct=yt(b,D,G,W,O);Vt.setMaterial(W,vt);let Bt=G.index,jt=1;if(W.wireframe===!0){if(Bt=at.getWireframeAttribute(G),Bt===void 0)return;jt=2}let Qt=G.drawRange,Dt=G.attributes.position,oe=Qt.start*jt,ue=(Qt.start+Qt.count)*jt;ht!==null&&(oe=Math.max(oe,ht.start*jt),ue=Math.min(ue,(ht.start+ht.count)*jt)),Bt!==null?(oe=Math.max(oe,0),ue=Math.min(ue,Bt.count)):Dt!=null&&(oe=Math.max(oe,0),ue=Math.min(ue,Dt.count));let me=ue-oe;if(me<0||me===1/0)return;Gt.setup(O,W,Ct,G,Bt);let Oe,ae=It;if(Bt!==null&&(Oe=j.get(Bt),ae=ne,ae.setIndex(Oe)),O.isMesh)W.wireframe===!0?(Vt.setLineWidth(W.wireframeLinewidth*le()),ae.setMode(N.LINES)):ae.setMode(N.TRIANGLES);else if(O.isLine){let Ht=W.linewidth;Ht===void 0&&(Ht=1),Vt.setLineWidth(Ht*le()),O.isLineSegments?ae.setMode(N.LINES):O.isLineLoop?ae.setMode(N.LINE_LOOP):ae.setMode(N.LINE_STRIP)}else O.isPoints?ae.setMode(N.POINTS):O.isSprite&&ae.setMode(N.TRIANGLES);if(O.isBatchedMesh)if(O._multiDrawInstances!==null)ae.renderMultiDrawInstances(O._multiDrawStarts,O._multiDrawCounts,O._multiDrawCount,O._multiDrawInstances);else if(Yt.get("WEBGL_multi_draw"))ae.renderMultiDraw(O._multiDrawStarts,O._multiDrawCounts,O._multiDrawCount);else{let Ht=O._multiDrawStarts,an=O._multiDrawCounts,pe=O._multiDrawCount,yn=Bt?j.get(Bt).bytesPerElement:1,Ui=Ut.get(W).currentProgram.getUniforms();for(let cn=0;cn<pe;cn++)Ui.setValue(N,"_gl_DrawID",cn),ae.render(Ht[cn]/yn,an[cn])}else if(O.isInstancedMesh)ae.renderInstances(oe,me,O.count);else if(G.isInstancedBufferGeometry){let Ht=G._maxInstanceCount!==void 0?G._maxInstanceCount:1/0,an=Math.min(G.instanceCount,Ht);ae.renderInstances(oe,me,an)}else ae.render(oe,me)};function nt(b,D,G){b.transparent===!0&&b.side===en&&b.forceSinglePass===!1?(b.side=ke,b.needsUpdate=!0,ft(b,D,G),b.side=Dn,b.needsUpdate=!0,ft(b,D,G),b.side=en):ft(b,D,G)}this.compile=function(b,D,G=null){G===null&&(G=b),d=re.get(G),d.init(D),_.push(d),G.traverseVisible(function(O){O.isLight&&O.layers.test(D.layers)&&(d.pushLight(O),O.castShadow&&d.pushShadow(O))}),b!==G&&b.traverseVisible(function(O){O.isLight&&O.layers.test(D.layers)&&(d.pushLight(O),O.castShadow&&d.pushShadow(O))}),d.setupLights();let W=new Set;return b.traverse(function(O){if(!(O.isMesh||O.isPoints||O.isLine||O.isSprite))return;let ht=O.material;if(ht)if(Array.isArray(ht))for(let vt=0;vt<ht.length;vt++){let Ct=ht[vt];nt(Ct,G,O),W.add(Ct)}else nt(ht,G,O),W.add(ht)}),_.pop(),d=null,W},this.compileAsync=function(b,D,G=null){let W=this.compile(b,D,G);return new Promise(O=>{function ht(){if(W.forEach(function(vt){Ut.get(vt).currentProgram.isReady()&&W.delete(vt)}),W.size===0){O(b);return}setTimeout(ht,10)}Yt.get("KHR_parallel_shader_compile")!==null?ht():setTimeout(ht,10)})};let gt=null;function xt(b){gt&&gt(b)}function te(){Ze.stop()}function He(){Ze.start()}let Ze=new fh;Ze.setAnimationLoop(xt),typeof self<"u"&&Ze.setContext(self),this.setAnimationLoop=function(b){gt=b,q.setAnimationLoop(b),b===null?Ze.stop():Ze.start()},q.addEventListener("sessionstart",te),q.addEventListener("sessionend",He),this.render=function(b,D){if(D!==void 0&&D.isCamera!==!0){console.error("THREE.WebGLRenderer.render: camera is not an instance of THREE.Camera.");return}if(C===!0)return;if(b.matrixWorldAutoUpdate===!0&&b.updateMatrixWorld(),D.parent===null&&D.matrixWorldAutoUpdate===!0&&D.updateMatrixWorld(),q.enabled===!0&&q.isPresenting===!0&&(q.cameraAutoUpdate===!0&&q.updateCamera(D),D=q.getCamera()),b.isScene===!0&&b.onBeforeRender(v,b,D,P),d=re.get(b,_.length),d.init(D),_.push(d),Pt.multiplyMatrices(D.projectionMatrix,D.matrixWorldInverse),H.setFromProjectionMatrix(Pt),pt=this.localClippingEnabled,st=lt.init(this.clippingPlanes,pt),m=Rt.get(b,x.length),m.init(),x.push(m),q.enabled===!0&&q.isPresenting===!0){let ht=v.xr.getDepthSensingMesh();ht!==null&&fe(ht,D,-1/0,v.sortObjects)}fe(b,D,0,v.sortObjects),m.finish(),v.sortObjects===!0&&m.sort(Q,ut),Jt=q.enabled===!1||q.isPresenting===!1||q.hasDepthSensing()===!1,Jt&&Nt.addToRenderList(m,b),this.info.render.frame++,st===!0&&lt.beginShadows();let G=d.state.shadowsArray;K.render(G,b,D),st===!0&&lt.endShadows(),this.info.autoReset===!0&&this.info.reset();let W=m.opaque,O=m.transmissive;if(d.setupLights(),D.isArrayCamera){let ht=D.cameras;if(O.length>0)for(let vt=0,Ct=ht.length;vt<Ct;vt++){let Bt=ht[vt];kn(W,O,b,Bt)}Jt&&Nt.render(b);for(let vt=0,Ct=ht.length;vt<Ct;vt++){let Bt=ht[vt];zn(m,b,Bt,Bt.viewport)}}else O.length>0&&kn(W,O,b,D),Jt&&Nt.render(b),zn(m,b,D);P!==null&&(T.updateMultisampleRenderTarget(P),T.updateRenderTargetMipmap(P)),b.isScene===!0&&b.onAfterRender(v,b,D),Gt.resetDefaultState(),E=-1,M=null,_.pop(),_.length>0?(d=_[_.length-1],st===!0&&lt.setGlobalState(v.clippingPlanes,d.state.camera)):d=null,x.pop(),x.length>0?m=x[x.length-1]:m=null};function fe(b,D,G,W){if(b.visible===!1)return;if(b.layers.test(D.layers)){if(b.isGroup)G=b.renderOrder;else if(b.isLOD)b.autoUpdate===!0&&b.update(D);else if(b.isLight)d.pushLight(b),b.castShadow&&d.pushShadow(b);else if(b.isSprite){if(!b.frustumCulled||H.intersectsSprite(b)){W&&Ot.setFromMatrixPosition(b.matrixWorld).applyMatrix4(Pt);let vt=J.update(b),Ct=b.material;Ct.visible&&m.push(b,vt,Ct,G,Ot.z,null)}}else if((b.isMesh||b.isLine||b.isPoints)&&(!b.frustumCulled||H.intersectsObject(b))){let vt=J.update(b),Ct=b.material;if(W&&(b.boundingSphere!==void 0?(b.boundingSphere===null&&b.computeBoundingSphere(),Ot.copy(b.boundingSphere.center)):(vt.boundingSphere===null&&vt.computeBoundingSphere(),Ot.copy(vt.boundingSphere.center)),Ot.applyMatrix4(b.matrixWorld).applyMatrix4(Pt)),Array.isArray(Ct)){let Bt=vt.groups;for(let jt=0,Qt=Bt.length;jt<Qt;jt++){let Dt=Bt[jt],oe=Ct[Dt.materialIndex];oe&&oe.visible&&m.push(b,vt,oe,G,Ot.z,Dt)}}else Ct.visible&&m.push(b,vt,Ct,G,Ot.z,null)}}let ht=b.children;for(let vt=0,Ct=ht.length;vt<Ct;vt++)fe(ht[vt],D,G,W)}function zn(b,D,G,W){let O=b.opaque,ht=b.transmissive,vt=b.transparent;d.setupLightsView(G),st===!0&&lt.setGlobalState(v.clippingPlanes,G),W&&Vt.viewport(R.copy(W)),O.length>0&&Z(O,D,G),ht.length>0&&Z(ht,D,G),vt.length>0&&Z(vt,D,G),Vt.buffers.depth.setTest(!0),Vt.buffers.depth.setMask(!0),Vt.buffers.color.setMask(!0),Vt.setPolygonOffset(!1)}function kn(b,D,G,W){if((G.isScene===!0?G.overrideMaterial:null)!==null)return;d.state.transmissionRenderTarget[W.id]===void 0&&(d.state.transmissionRenderTarget[W.id]=new En(1,1,{generateMipmaps:!0,type:Yt.has("EXT_color_buffer_half_float")||Yt.has("EXT_color_buffer_float")?Ys:Jn,minFilter:qn,samples:4,stencilBuffer:r,resolveDepthBuffer:!1,resolveStencilBuffer:!1,colorSpace:he.workingColorSpace}));let ht=d.state.transmissionRenderTarget[W.id],vt=W.viewport||R;ht.setSize(vt.z,vt.w);let Ct=v.getRenderTarget();v.setRenderTarget(ht),v.getClearColor(z),k=v.getClearAlpha(),k<1&&v.setClearColor(16777215,.5),v.clear(),Jt&&Nt.render(G);let Bt=v.toneMapping;v.toneMapping=fi;let jt=W.viewport;if(W.viewport!==void 0&&(W.viewport=void 0),d.setupLightsView(W),st===!0&&lt.setGlobalState(v.clippingPlanes,W),Z(b,G,W),T.updateMultisampleRenderTarget(ht),T.updateRenderTargetMipmap(ht),Yt.has("WEBGL_multisampled_render_to_texture")===!1){let Qt=!1;for(let Dt=0,oe=D.length;Dt<oe;Dt++){let ue=D[Dt],me=ue.object,Oe=ue.geometry,ae=ue.material,Ht=ue.group;if(ae.side===en&&me.layers.test(W.layers)){let an=ae.side;ae.side=ke,ae.needsUpdate=!0,it(me,G,W,Oe,ae,Ht),ae.side=an,ae.needsUpdate=!0,Qt=!0}}Qt===!0&&(T.updateMultisampleRenderTarget(ht),T.updateRenderTargetMipmap(ht))}v.setRenderTarget(Ct),v.setClearColor(z,k),jt!==void 0&&(W.viewport=jt),v.toneMapping=Bt}function Z(b,D,G){let W=D.isScene===!0?D.overrideMaterial:null;for(let O=0,ht=b.length;O<ht;O++){let vt=b[O],Ct=vt.object,Bt=vt.geometry,jt=W===null?vt.material:W,Qt=vt.group;Ct.layers.test(G.layers)&&it(Ct,D,G,Bt,jt,Qt)}}function it(b,D,G,W,O,ht){b.onBeforeRender(v,D,G,W,O,ht),b.modelViewMatrix.multiplyMatrices(G.matrixWorldInverse,b.matrixWorld),b.normalMatrix.getNormalMatrix(b.modelViewMatrix),O.onBeforeRender(v,D,G,W,b,ht),O.transparent===!0&&O.side===en&&O.forceSinglePass===!1?(O.side=ke,O.needsUpdate=!0,v.renderBufferDirect(G,D,W,O,b,ht),O.side=Dn,O.needsUpdate=!0,v.renderBufferDirect(G,D,W,O,b,ht),O.side=en):v.renderBufferDirect(G,D,W,O,b,ht),b.onAfterRender(v,D,G,W,O,ht)}function ft(b,D,G){D.isScene!==!0&&(D=Xt);let W=Ut.get(b),O=d.state.lights,ht=d.state.shadowsArray,vt=O.state.version,Ct=Ft.getParameters(b,O.state,ht,D,G),Bt=Ft.getProgramCacheKey(Ct),jt=W.programs;W.environment=b.isMeshStandardMaterial?D.environment:null,W.fog=D.fog,W.envMap=(b.isMeshStandardMaterial?X:S).get(b.envMap||W.environment),W.envMapRotation=W.environment!==null&&b.envMap===null?D.environmentRotation:b.envMapRotation,jt===void 0&&(b.addEventListener("dispose",et),jt=new Map,W.programs=jt);let Qt=jt.get(Bt);if(Qt!==void 0){if(W.currentProgram===Qt&&W.lightsStateVersion===vt)return mt(b,Ct),Qt}else Ct.uniforms=Ft.getUniforms(b),b.onBeforeCompile(Ct,v),Qt=Ft.acquireProgram(Ct,Bt),jt.set(Bt,Qt),W.uniforms=Ct.uniforms;let Dt=W.uniforms;return(!b.isShaderMaterial&&!b.isRawShaderMaterial||b.clipping===!0)&&(Dt.clippingPlanes=lt.uniform),mt(b,Ct),W.needsLights=kt(b),W.lightsStateVersion=vt,W.needsLights&&(Dt.ambientLightColor.value=O.state.ambient,Dt.lightProbe.value=O.state.probe,Dt.directionalLights.value=O.state.directional,Dt.directionalLightShadows.value=O.state.directionalShadow,Dt.spotLights.value=O.state.spot,Dt.spotLightShadows.value=O.state.spotShadow,Dt.rectAreaLights.value=O.state.rectArea,Dt.ltc_1.value=O.state.rectAreaLTC1,Dt.ltc_2.value=O.state.rectAreaLTC2,Dt.pointLights.value=O.state.point,Dt.pointLightShadows.value=O.state.pointShadow,Dt.hemisphereLights.value=O.state.hemi,Dt.directionalShadowMap.value=O.state.directionalShadowMap,Dt.directionalShadowMatrix.value=O.state.directionalShadowMatrix,Dt.spotShadowMap.value=O.state.spotShadowMap,Dt.spotLightMatrix.value=O.state.spotLightMatrix,Dt.spotLightMap.value=O.state.spotLightMap,Dt.pointShadowMap.value=O.state.pointShadowMap,Dt.pointShadowMatrix.value=O.state.pointShadowMatrix),W.currentProgram=Qt,W.uniformsList=null,Qt}function rt(b){if(b.uniformsList===null){let D=b.currentProgram.getUniforms();b.uniformsList=ss.seqWithValue(D.seq,b.uniforms)}return b.uniformsList}function mt(b,D){let G=Ut.get(b);G.outputColorSpace=D.outputColorSpace,G.batching=D.batching,G.batchingColor=D.batchingColor,G.instancing=D.instancing,G.instancingColor=D.instancingColor,G.instancingMorph=D.instancingMorph,G.skinning=D.skinning,G.morphTargets=D.morphTargets,G.morphNormals=D.morphNormals,G.morphColors=D.morphColors,G.morphTargetsCount=D.morphTargetsCount,G.numClippingPlanes=D.numClippingPlanes,G.numIntersection=D.numClipIntersection,G.vertexAlphas=D.vertexAlphas,G.vertexTangents=D.vertexTangents,G.toneMapping=D.toneMapping}function yt(b,D,G,W,O){D.isScene!==!0&&(D=Xt),T.resetTextureUnits();let ht=D.fog,vt=W.isMeshStandardMaterial?D.environment:null,Ct=P===null?v.outputColorSpace:P.isXRRenderTarget===!0?P.texture.colorSpace:_s,Bt=(W.isMeshStandardMaterial?X:S).get(W.envMap||vt),jt=W.vertexColors===!0&&!!G.attributes.color&&G.attributes.color.itemSize===4,Qt=!!G.attributes.tangent&&(!!W.normalMap||W.anisotropy>0),Dt=!!G.morphAttributes.position,oe=!!G.morphAttributes.normal,ue=!!G.morphAttributes.color,me=fi;W.toneMapped&&(P===null||P.isXRRenderTarget===!0)&&(me=v.toneMapping);let Oe=G.morphAttributes.position||G.morphAttributes.normal||G.morphAttributes.color,ae=Oe!==void 0?Oe.length:0,Ht=Ut.get(W),an=d.state.lights;if(st===!0&&(pt===!0||b!==M)){let mn=b===M&&W.id===E;lt.setState(W,b,mn)}let pe=!1;W.version===Ht.__version?(Ht.needsLights&&Ht.lightsStateVersion!==an.state.version||Ht.outputColorSpace!==Ct||O.isBatchedMesh&&Ht.batching===!1||!O.isBatchedMesh&&Ht.batching===!0||O.isBatchedMesh&&Ht.batchingColor===!0&&O.colorTexture===null||O.isBatchedMesh&&Ht.batchingColor===!1&&O.colorTexture!==null||O.isInstancedMesh&&Ht.instancing===!1||!O.isInstancedMesh&&Ht.instancing===!0||O.isSkinnedMesh&&Ht.skinning===!1||!O.isSkinnedMesh&&Ht.skinning===!0||O.isInstancedMesh&&Ht.instancingColor===!0&&O.instanceColor===null||O.isInstancedMesh&&Ht.instancingColor===!1&&O.instanceColor!==null||O.isInstancedMesh&&Ht.instancingMorph===!0&&O.morphTexture===null||O.isInstancedMesh&&Ht.instancingMorph===!1&&O.morphTexture!==null||Ht.envMap!==Bt||W.fog===!0&&Ht.fog!==ht||Ht.numClippingPlanes!==void 0&&(Ht.numClippingPlanes!==lt.numPlanes||Ht.numIntersection!==lt.numIntersection)||Ht.vertexAlphas!==jt||Ht.vertexTangents!==Qt||Ht.morphTargets!==Dt||Ht.morphNormals!==oe||Ht.morphColors!==ue||Ht.toneMapping!==me||Ht.morphTargetsCount!==ae)&&(pe=!0):(pe=!0,Ht.__version=W.version);let yn=Ht.currentProgram;pe===!0&&(yn=ft(W,D,O));let Ui=!1,cn=!1,Ss=!1,Se=yn.getUniforms(),Pn=Ht.uniforms;if(Vt.useProgram(yn.program)&&(Ui=!0,cn=!0,Ss=!0),W.id!==E&&(E=W.id,cn=!0),Ui||M!==b){Vt.buffers.depth.getReversed()?(ct.copy(b.projectionMatrix),yd(ct),vd(ct),Se.setValue(N,"projectionMatrix",ct)):Se.setValue(N,"projectionMatrix",b.projectionMatrix),Se.setValue(N,"viewMatrix",b.matrixWorldInverse);let ni=Se.map.cameraPosition;ni!==void 0&&ni.setValue(N,At.setFromMatrixPosition(b.matrixWorld)),qt.logarithmicDepthBuffer&&Se.setValue(N,"logDepthBufFC",2/(Math.log(b.far+1)/Math.LN2)),(W.isMeshPhongMaterial||W.isMeshToonMaterial||W.isMeshLambertMaterial||W.isMeshBasicMaterial||W.isMeshStandardMaterial||W.isShaderMaterial)&&Se.setValue(N,"isOrthographic",b.isOrthographicCamera===!0),M!==b&&(M=b,cn=!0,Ss=!0)}if(O.isSkinnedMesh){Se.setOptional(N,O,"bindMatrix"),Se.setOptional(N,O,"bindMatrixInverse");let mn=O.skeleton;mn&&(mn.boneTexture===null&&mn.computeBoneTexture(),Se.setValue(N,"boneTexture",mn.boneTexture,T))}O.isBatchedMesh&&(Se.setOptional(N,O,"batchingTexture"),Se.setValue(N,"batchingTexture",O._matricesTexture,T),Se.setOptional(N,O,"batchingIdTexture"),Se.setValue(N,"batchingIdTexture",O._indirectTexture,T),Se.setOptional(N,O,"batchingColorTexture"),O._colorsTexture!==null&&Se.setValue(N,"batchingColorTexture",O._colorsTexture,T));let bs=G.morphAttributes;if((bs.position!==void 0||bs.normal!==void 0||bs.color!==void 0)&&Zt.update(O,G,yn),(cn||Ht.receiveShadow!==O.receiveShadow)&&(Ht.receiveShadow=O.receiveShadow,Se.setValue(N,"receiveShadow",O.receiveShadow)),W.isMeshGouraudMaterial&&W.envMap!==null&&(Pn.envMap.value=Bt,Pn.flipEnvMap.value=Bt.isCubeTexture&&Bt.isRenderTargetTexture===!1?-1:1),W.isMeshStandardMaterial&&W.envMap===null&&D.environment!==null&&(Pn.envMapIntensity.value=D.environmentIntensity),cn&&(Se.setValue(N,"toneMappingExposure",v.toneMappingExposure),Ht.needsLights&&Tt(Pn,Ss),ht&&W.fog===!0&&St.refreshFogUniforms(Pn,ht),St.refreshMaterialUniforms(Pn,W,B,Y,d.state.transmissionRenderTarget[b.id]),ss.upload(N,rt(Ht),Pn,T)),W.isShaderMaterial&&W.uniformsNeedUpdate===!0&&(ss.upload(N,rt(Ht),Pn,T),W.uniformsNeedUpdate=!1),W.isSpriteMaterial&&Se.setValue(N,"center",O.center),Se.setValue(N,"modelViewMatrix",O.modelViewMatrix),Se.setValue(N,"normalMatrix",O.normalMatrix),Se.setValue(N,"modelMatrix",O.matrixWorld),W.isShaderMaterial||W.isRawShaderMaterial){let mn=W.uniformsGroups;for(let ni=0,ii=mn.length;ni<ii;ni++){let Gc=mn[ni];L.update(Gc,yn),L.bind(Gc,yn)}}return yn}function Tt(b,D){b.ambientLightColor.needsUpdate=D,b.lightProbe.needsUpdate=D,b.directionalLights.needsUpdate=D,b.directionalLightShadows.needsUpdate=D,b.pointLights.needsUpdate=D,b.pointLightShadows.needsUpdate=D,b.spotLights.needsUpdate=D,b.spotLightShadows.needsUpdate=D,b.rectAreaLights.needsUpdate=D,b.hemisphereLights.needsUpdate=D}function kt(b){return b.isMeshLambertMaterial||b.isMeshToonMaterial||b.isMeshPhongMaterial||b.isMeshStandardMaterial||b.isShadowMaterial||b.isShaderMaterial&&b.lights===!0}this.getActiveCubeFace=function(){return w},this.getActiveMipmapLevel=function(){return A},this.getRenderTarget=function(){return P},this.setRenderTargetTextures=function(b,D,G){Ut.get(b.texture).__webglTexture=D,Ut.get(b.depthTexture).__webglTexture=G;let W=Ut.get(b);W.__hasExternalTextures=!0,W.__autoAllocateDepthBuffer=G===void 0,W.__autoAllocateDepthBuffer||Yt.has("WEBGL_multisampled_render_to_texture")===!0&&(console.warn("THREE.WebGLRenderer: Render-to-texture extension was disabled because an external texture was provided"),W.__useRenderToTexture=!1)},this.setRenderTargetFramebuffer=function(b,D){let G=Ut.get(b);G.__webglFramebuffer=D,G.__useDefaultFramebuffer=D===void 0},this.setRenderTarget=function(b,D=0,G=0){P=b,w=D,A=G;let W=!0,O=null,ht=!1,vt=!1;if(b){let Bt=Ut.get(b);if(Bt.__useDefaultFramebuffer!==void 0)Vt.bindFramebuffer(N.FRAMEBUFFER,null),W=!1;else if(Bt.__webglFramebuffer===void 0)T.setupRenderTarget(b);else if(Bt.__hasExternalTextures)T.rebindTextures(b,Ut.get(b.texture).__webglTexture,Ut.get(b.depthTexture).__webglTexture);else if(b.depthBuffer){let Dt=b.depthTexture;if(Bt.__boundDepthTexture!==Dt){if(Dt!==null&&Ut.has(Dt)&&(b.width!==Dt.image.width||b.height!==Dt.image.height))throw new Error("WebGLRenderTarget: Attached DepthTexture is initialized to the incorrect size.");T.setupDepthRenderbuffer(b)}}let jt=b.texture;(jt.isData3DTexture||jt.isDataArrayTexture||jt.isCompressedArrayTexture)&&(vt=!0);let Qt=Ut.get(b).__webglFramebuffer;b.isWebGLCubeRenderTarget?(Array.isArray(Qt[D])?O=Qt[D][G]:O=Qt[D],ht=!0):b.samples>0&&T.useMultisampledRTT(b)===!1?O=Ut.get(b).__webglMultisampledFramebuffer:Array.isArray(Qt)?O=Qt[G]:O=Qt,R.copy(b.viewport),F.copy(b.scissor),V=b.scissorTest}else R.copy(Et).multiplyScalar(B).floor(),F.copy(zt).multiplyScalar(B).floor(),V=$t;if(Vt.bindFramebuffer(N.FRAMEBUFFER,O)&&W&&Vt.drawBuffers(b,O),Vt.viewport(R),Vt.scissor(F),Vt.setScissorTest(V),ht){let Bt=Ut.get(b.texture);N.framebufferTexture2D(N.FRAMEBUFFER,N.COLOR_ATTACHMENT0,N.TEXTURE_CUBE_MAP_POSITIVE_X+D,Bt.__webglTexture,G)}else if(vt){let Bt=Ut.get(b.texture),jt=D||0;N.framebufferTextureLayer(N.FRAMEBUFFER,N.COLOR_ATTACHMENT0,Bt.__webglTexture,G||0,jt)}E=-1},this.readRenderTargetPixels=function(b,D,G,W,O,ht,vt){if(!(b&&b.isWebGLRenderTarget)){console.error("THREE.WebGLRenderer.readRenderTargetPixels: renderTarget is not THREE.WebGLRenderTarget.");return}let Ct=Ut.get(b).__webglFramebuffer;if(b.isWebGLCubeRenderTarget&&vt!==void 0&&(Ct=Ct[vt]),Ct){Vt.bindFramebuffer(N.FRAMEBUFFER,Ct);try{let Bt=b.texture,jt=Bt.format,Qt=Bt.type;if(!qt.textureFormatReadable(jt)){console.error("THREE.WebGLRenderer.readRenderTargetPixels: renderTarget is not in RGBA or implementation defined format.");return}if(!qt.textureTypeReadable(Qt)){console.error("THREE.WebGLRenderer.readRenderTargetPixels: renderTarget is not in UnsignedByteType or implementation defined type.");return}D>=0&&D<=b.width-W&&G>=0&&G<=b.height-O&&N.readPixels(D,G,W,O,Kt.convert(jt),Kt.convert(Qt),ht)}finally{let Bt=P!==null?Ut.get(P).__webglFramebuffer:null;Vt.bindFramebuffer(N.FRAMEBUFFER,Bt)}}},this.readRenderTargetPixelsAsync=async function(b,D,G,W,O,ht,vt){if(!(b&&b.isWebGLRenderTarget))throw new Error("THREE.WebGLRenderer.readRenderTargetPixels: renderTarget is not THREE.WebGLRenderTarget.");let Ct=Ut.get(b).__webglFramebuffer;if(b.isWebGLCubeRenderTarget&&vt!==void 0&&(Ct=Ct[vt]),Ct){let Bt=b.texture,jt=Bt.format,Qt=Bt.type;if(!qt.textureFormatReadable(jt))throw new Error("THREE.WebGLRenderer.readRenderTargetPixelsAsync: renderTarget is not in RGBA or implementation defined format.");if(!qt.textureTypeReadable(Qt))throw new Error("THREE.WebGLRenderer.readRenderTargetPixelsAsync: renderTarget is not in UnsignedByteType or implementation defined type.");if(D>=0&&D<=b.width-W&&G>=0&&G<=b.height-O){Vt.bindFramebuffer(N.FRAMEBUFFER,Ct);let Dt=N.createBuffer();N.bindBuffer(N.PIXEL_PACK_BUFFER,Dt),N.bufferData(N.PIXEL_PACK_BUFFER,ht.byteLength,N.STREAM_READ),N.readPixels(D,G,W,O,Kt.convert(jt),Kt.convert(Qt),0);let oe=P!==null?Ut.get(P).__webglFramebuffer:null;Vt.bindFramebuffer(N.FRAMEBUFFER,oe);let ue=N.fenceSync(N.SYNC_GPU_COMMANDS_COMPLETE,0);return N.flush(),await xd(N,ue,4),N.bindBuffer(N.PIXEL_PACK_BUFFER,Dt),N.getBufferSubData(N.PIXEL_PACK_BUFFER,0,ht),N.deleteBuffer(Dt),N.deleteSync(ue),ht}else throw new Error("THREE.WebGLRenderer.readRenderTargetPixelsAsync: requested read bounds are out of range.")}},this.copyFramebufferToTexture=function(b,D=null,G=0){b.isTexture!==!0&&(Ds("WebGLRenderer: copyFramebufferToTexture function signature has changed."),D=arguments[0]||null,b=arguments[1]);let W=Math.pow(2,-G),O=Math.floor(b.image.width*W),ht=Math.floor(b.image.height*W),vt=D!==null?D.x:0,Ct=D!==null?D.y:0;T.setTexture2D(b,0),N.copyTexSubImage2D(N.TEXTURE_2D,G,0,0,vt,Ct,O,ht),Vt.unbindTexture()},this.copyTextureToTexture=function(b,D,G=null,W=null,O=0){b.isTexture!==!0&&(Ds("WebGLRenderer: copyTextureToTexture function signature has changed."),W=arguments[0]||null,b=arguments[1],D=arguments[2],O=arguments[3]||0,G=null);let ht,vt,Ct,Bt,jt,Qt,Dt,oe,ue,me=b.isCompressedTexture?b.mipmaps[O]:b.image;G!==null?(ht=G.max.x-G.min.x,vt=G.max.y-G.min.y,Ct=G.isBox3?G.max.z-G.min.z:1,Bt=G.min.x,jt=G.min.y,Qt=G.isBox3?G.min.z:0):(ht=me.width,vt=me.height,Ct=me.depth||1,Bt=0,jt=0,Qt=0),W!==null?(Dt=W.x,oe=W.y,ue=W.z):(Dt=0,oe=0,ue=0);let Oe=Kt.convert(D.format),ae=Kt.convert(D.type),Ht;D.isData3DTexture?(T.setTexture3D(D,0),Ht=N.TEXTURE_3D):D.isDataArrayTexture||D.isCompressedArrayTexture?(T.setTexture2DArray(D,0),Ht=N.TEXTURE_2D_ARRAY):(T.setTexture2D(D,0),Ht=N.TEXTURE_2D),N.pixelStorei(N.UNPACK_FLIP_Y_WEBGL,D.flipY),N.pixelStorei(N.UNPACK_PREMULTIPLY_ALPHA_WEBGL,D.premultiplyAlpha),N.pixelStorei(N.UNPACK_ALIGNMENT,D.unpackAlignment);let an=N.getParameter(N.UNPACK_ROW_LENGTH),pe=N.getParameter(N.UNPACK_IMAGE_HEIGHT),yn=N.getParameter(N.UNPACK_SKIP_PIXELS),Ui=N.getParameter(N.UNPACK_SKIP_ROWS),cn=N.getParameter(N.UNPACK_SKIP_IMAGES);N.pixelStorei(N.UNPACK_ROW_LENGTH,me.width),N.pixelStorei(N.UNPACK_IMAGE_HEIGHT,me.height),N.pixelStorei(N.UNPACK_SKIP_PIXELS,Bt),N.pixelStorei(N.UNPACK_SKIP_ROWS,jt),N.pixelStorei(N.UNPACK_SKIP_IMAGES,Qt);let Ss=b.isDataArrayTexture||b.isData3DTexture,Se=D.isDataArrayTexture||D.isData3DTexture;if(b.isRenderTargetTexture||b.isDepthTexture){let Pn=Ut.get(b),bs=Ut.get(D),mn=Ut.get(Pn.__renderTarget),ni=Ut.get(bs.__renderTarget);Vt.bindFramebuffer(N.READ_FRAMEBUFFER,mn.__webglFramebuffer),Vt.bindFramebuffer(N.DRAW_FRAMEBUFFER,ni.__webglFramebuffer);for(let ii=0;ii<Ct;ii++)Ss&&N.framebufferTextureLayer(N.READ_FRAMEBUFFER,N.COLOR_ATTACHMENT0,Ut.get(b).__webglTexture,O,Qt+ii),b.isDepthTexture?(Se&&N.framebufferTextureLayer(N.DRAW_FRAMEBUFFER,N.COLOR_ATTACHMENT0,Ut.get(D).__webglTexture,O,ue+ii),N.blitFramebuffer(Bt,jt,ht,vt,Dt,oe,ht,vt,N.DEPTH_BUFFER_BIT,N.NEAREST)):Se?N.copyTexSubImage3D(Ht,O,Dt,oe,ue+ii,Bt,jt,ht,vt):N.copyTexSubImage2D(Ht,O,Dt,oe,ue+ii,Bt,jt,ht,vt);Vt.bindFramebuffer(N.READ_FRAMEBUFFER,null),Vt.bindFramebuffer(N.DRAW_FRAMEBUFFER,null)}else Se?b.isDataTexture||b.isData3DTexture?N.texSubImage3D(Ht,O,Dt,oe,ue,ht,vt,Ct,Oe,ae,me.data):D.isCompressedArrayTexture?N.compressedTexSubImage3D(Ht,O,Dt,oe,ue,ht,vt,Ct,Oe,me.data):N.texSubImage3D(Ht,O,Dt,oe,ue,ht,vt,Ct,Oe,ae,me):b.isDataTexture?N.texSubImage2D(N.TEXTURE_2D,O,Dt,oe,ht,vt,Oe,ae,me.data):b.isCompressedTexture?N.compressedTexSubImage2D(N.TEXTURE_2D,O,Dt,oe,me.width,me.height,Oe,me.data):N.texSubImage2D(N.TEXTURE_2D,O,Dt,oe,ht,vt,Oe,ae,me);N.pixelStorei(N.UNPACK_ROW_LENGTH,an),N.pixelStorei(N.UNPACK_IMAGE_HEIGHT,pe),N.pixelStorei(N.UNPACK_SKIP_PIXELS,yn),N.pixelStorei(N.UNPACK_SKIP_ROWS,Ui),N.pixelStorei(N.UNPACK_SKIP_IMAGES,cn),O===0&&D.generateMipmaps&&N.generateMipmap(Ht),Vt.unbindTexture()},this.copyTextureToTexture3D=function(b,D,G=null,W=null,O=0){return b.isTexture!==!0&&(Ds("WebGLRenderer: copyTextureToTexture3D function signature has changed."),G=arguments[0]||null,W=arguments[1]||null,b=arguments[2],D=arguments[3],O=arguments[4]||0),Ds('WebGLRenderer: copyTextureToTexture3D function has been deprecated. Use "copyTextureToTexture" instead.'),this.copyTextureToTexture(b,D,G,W,O)},this.initRenderTarget=function(b){Ut.get(b).__webglFramebuffer===void 0&&T.setupRenderTarget(b)},this.initTexture=function(b){b.isCubeTexture?T.setTextureCube(b,0):b.isData3DTexture?T.setTexture3D(b,0):b.isDataArrayTexture||b.isCompressedArrayTexture?T.setTexture2DArray(b,0):T.setTexture2D(b,0),Vt.unbindTexture()},this.resetState=function(){w=0,A=0,P=null,Vt.reset(),Gt.reset()},typeof __THREE_DEVTOOLS__<"u"&&__THREE_DEVTOOLS__.dispatchEvent(new CustomEvent("observe",{detail:this}))}get coordinateSystem(){return $n}get outputColorSpace(){return this._outputColorSpace}set outputColorSpace(t){this._outputColorSpace=t;let e=this.getContext();e.drawingBufferColorspace=he._getDrawingBufferColorSpace(t),e.unpackColorSpace=he._getUnpackColorSpace()}},fs=class i{constructor(t,e=25e-5){this.isFogExp2=!0,this.name="",this.color=new _t(t),this.density=e}clone(){return new i(this.color,this.density)}toJSON(){return{type:"FogExp2",name:this.name,color:this.color.getHex(),density:this.density}}};var ps=class extends Re{constructor(){super(),this.isScene=!0,this.type="Scene",this.background=null,this.environment=null,this.fog=null,this.backgroundBlurriness=0,this.backgroundIntensity=1,this.backgroundRotation=new sn,this.environmentIntensity=1,this.environmentRotation=new sn,this.overrideMaterial=null,typeof __THREE_DEVTOOLS__<"u"&&__THREE_DEVTOOLS__.dispatchEvent(new CustomEvent("observe",{detail:this}))}copy(t,e){return super.copy(t,e),t.background!==null&&(this.background=t.background.clone()),t.environment!==null&&(this.environment=t.environment.clone()),t.fog!==null&&(this.fog=t.fog.clone()),this.backgroundBlurriness=t.backgroundBlurriness,this.backgroundIntensity=t.backgroundIntensity,this.backgroundRotation.copy(t.backgroundRotation),this.environmentIntensity=t.environmentIntensity,this.environmentRotation.copy(t.environmentRotation),t.overrideMaterial!==null&&(this.overrideMaterial=t.overrideMaterial.clone()),this.matrixAutoUpdate=t.matrixAutoUpdate,this}toJSON(t){let e=super.toJSON(t);return this.fog!==null&&(e.object.fog=this.fog.toJSON()),this.backgroundBlurriness>0&&(e.object.backgroundBlurriness=this.backgroundBlurriness),this.backgroundIntensity!==1&&(e.object.backgroundIntensity=this.backgroundIntensity),e.object.backgroundRotation=this.backgroundRotation.toArray(),this.environmentIntensity!==1&&(e.object.environmentIntensity=this.environmentIntensity),e.object.environmentRotation=this.environmentRotation.toArray(),e}},Wr=class{constructor(t,e){this.isInterleavedBuffer=!0,this.array=t,this.stride=e,this.count=t!==void 0?t.length/e:0,this.usage=Na,this.updateRanges=[],this.version=0,this.uuid=Zn()}onUploadCallback(){}set needsUpdate(t){t===!0&&this.version++}setUsage(t){return this.usage=t,this}addUpdateRange(t,e){this.updateRanges.push({start:t,count:e})}clearUpdateRanges(){this.updateRanges.length=0}copy(t){return this.array=new t.array.constructor(t.array),this.count=t.count,this.stride=t.stride,this.usage=t.usage,this}copyAt(t,e,n){t*=this.stride,n*=e.stride;for(let s=0,r=this.stride;s<r;s++)this.array[t+s]=e.array[n+s];return this}set(t,e=0){return this.array.set(t,e),this}clone(t){t.arrayBuffers===void 0&&(t.arrayBuffers={}),this.array.buffer._uuid===void 0&&(this.array.buffer._uuid=Zn()),t.arrayBuffers[this.array.buffer._uuid]===void 0&&(t.arrayBuffers[this.array.buffer._uuid]=this.array.slice(0).buffer);let e=new this.array.constructor(t.arrayBuffers[this.array.buffer._uuid]),n=new this.constructor(e,this.stride);return n.setUsage(this.usage),n}onUpload(t){return this.onUploadCallback=t,this}toJSON(t){return t.arrayBuffers===void 0&&(t.arrayBuffers={}),this.array.buffer._uuid===void 0&&(this.array.buffer._uuid=Zn()),t.arrayBuffers[this.array.buffer._uuid]===void 0&&(t.arrayBuffers[this.array.buffer._uuid]=Array.from(new Uint32Array(this.array.buffer))),{uuid:this.uuid,buffer:this.array.buffer._uuid,type:this.array.constructor.name,stride:this.stride}}},Ke=new U,Hs=class i{constructor(t,e,n,s=!1){this.isInterleavedBufferAttribute=!0,this.name="",this.data=t,this.itemSize=e,this.offset=n,this.normalized=s}get count(){return this.data.count}get array(){return this.data.array}set needsUpdate(t){this.data.needsUpdate=t}applyMatrix4(t){for(let e=0,n=this.data.count;e<n;e++)Ke.fromBufferAttribute(this,e),Ke.applyMatrix4(t),this.setXYZ(e,Ke.x,Ke.y,Ke.z);return this}applyNormalMatrix(t){for(let e=0,n=this.count;e<n;e++)Ke.fromBufferAttribute(this,e),Ke.applyNormalMatrix(t),this.setXYZ(e,Ke.x,Ke.y,Ke.z);return this}transformDirection(t){for(let e=0,n=this.count;e<n;e++)Ke.fromBufferAttribute(this,e),Ke.transformDirection(t),this.setXYZ(e,Ke.x,Ke.y,Ke.z);return this}getComponent(t,e){let n=this.array[t*this.data.stride+this.offset+e];return this.normalized&&(n=bn(n,this.array)),n}setComponent(t,e,n){return this.normalized&&(n=ge(n,this.array)),this.data.array[t*this.data.stride+this.offset+e]=n,this}setX(t,e){return this.normalized&&(e=ge(e,this.array)),this.data.array[t*this.data.stride+this.offset]=e,this}setY(t,e){return this.normalized&&(e=ge(e,this.array)),this.data.array[t*this.data.stride+this.offset+1]=e,this}setZ(t,e){return this.normalized&&(e=ge(e,this.array)),this.data.array[t*this.data.stride+this.offset+2]=e,this}setW(t,e){return this.normalized&&(e=ge(e,this.array)),this.data.array[t*this.data.stride+this.offset+3]=e,this}getX(t){let e=this.data.array[t*this.data.stride+this.offset];return this.normalized&&(e=bn(e,this.array)),e}getY(t){let e=this.data.array[t*this.data.stride+this.offset+1];return this.normalized&&(e=bn(e,this.array)),e}getZ(t){let e=this.data.array[t*this.data.stride+this.offset+2];return this.normalized&&(e=bn(e,this.array)),e}getW(t){let e=this.data.array[t*this.data.stride+this.offset+3];return this.normalized&&(e=bn(e,this.array)),e}setXY(t,e,n){return t=t*this.data.stride+this.offset,this.normalized&&(e=ge(e,this.array),n=ge(n,this.array)),this.data.array[t+0]=e,this.data.array[t+1]=n,this}setXYZ(t,e,n,s){return t=t*this.data.stride+this.offset,this.normalized&&(e=ge(e,this.array),n=ge(n,this.array),s=ge(s,this.array)),this.data.array[t+0]=e,this.data.array[t+1]=n,this.data.array[t+2]=s,this}setXYZW(t,e,n,s,r){return t=t*this.data.stride+this.offset,this.normalized&&(e=ge(e,this.array),n=ge(n,this.array),s=ge(s,this.array),r=ge(r,this.array)),this.data.array[t+0]=e,this.data.array[t+1]=n,this.data.array[t+2]=s,this.data.array[t+3]=r,this}clone(t){if(t===void 0){console.log("THREE.InterleavedBufferAttribute.clone(): Cloning an interleaved buffer attribute will de-interleave buffer data.");let e=[];for(let n=0;n<this.count;n++){let s=n*this.data.stride+this.offset;for(let r=0;r<this.itemSize;r++)e.push(this.data.array[s+r])}return new ve(new this.array.constructor(e),this.itemSize,this.normalized)}else return t.interleavedBuffers===void 0&&(t.interleavedBuffers={}),t.interleavedBuffers[this.data.uuid]===void 0&&(t.interleavedBuffers[this.data.uuid]=this.data.clone(t)),new i(t.interleavedBuffers[this.data.uuid],this.itemSize,this.offset,this.normalized)}toJSON(t){if(t===void 0){console.log("THREE.InterleavedBufferAttribute.toJSON(): Serializing an interleaved buffer attribute will de-interleave buffer data.");let e=[];for(let n=0;n<this.count;n++){let s=n*this.data.stride+this.offset;for(let r=0;r<this.itemSize;r++)e.push(this.data.array[s+r])}return{itemSize:this.itemSize,type:this.array.constructor.name,array:e,normalized:this.normalized}}else return t.interleavedBuffers===void 0&&(t.interleavedBuffers={}),t.interleavedBuffers[this.data.uuid]===void 0&&(t.interleavedBuffers[this.data.uuid]=this.data.toJSON(t)),{isInterleavedBufferAttribute:!0,itemSize:this.itemSize,data:this.data.uuid,offset:this.offset,normalized:this.normalized}}},Vs=class extends Nn{static get type(){return"SpriteMaterial"}constructor(t){super(),this.isSpriteMaterial=!0,this.color=new _t(16777215),this.map=null,this.alphaMap=null,this.rotation=0,this.sizeAttenuation=!0,this.transparent=!0,this.fog=!0,this.setValues(t)}copy(t){return super.copy(t),this.color.copy(t.color),this.map=t.map,this.alphaMap=t.alphaMap,this.rotation=t.rotation,this.sizeAttenuation=t.sizeAttenuation,this.fog=t.fog,this}},Zi,Cs=new U,Ki=new U,Ji=new U,ji=new Wt,Rs=new Wt,xh=new ce,yr=new U,Ps=new U,vr=new U,Gl=new Wt,$o=new Wt,Wl=new Wt,Xr=class extends Re{constructor(t=new Vs){if(super(),this.isSprite=!0,this.type="Sprite",Zi===void 0){Zi=new Ge;let e=new Float32Array([-.5,-.5,0,0,0,.5,-.5,0,1,0,.5,.5,0,1,1,-.5,.5,0,0,1]),n=new Wr(e,5);Zi.setIndex([0,1,2,0,2,3]),Zi.setAttribute("position",new Hs(n,3,0,!1)),Zi.setAttribute("uv",new Hs(n,2,3,!1))}this.geometry=Zi,this.material=t,this.center=new Wt(.5,.5)}raycast(t,e){t.camera===null&&console.error('THREE.Sprite: "Raycaster.camera" needs to be set in order to raycast against sprites.'),Ki.setFromMatrixScale(this.matrixWorld),xh.copy(t.camera.matrixWorld),this.modelViewMatrix.multiplyMatrices(t.camera.matrixWorldInverse,this.matrixWorld),Ji.setFromMatrixPosition(this.modelViewMatrix),t.camera.isPerspectiveCamera&&this.material.sizeAttenuation===!1&&Ki.multiplyScalar(-Ji.z);let n=this.material.rotation,s,r;n!==0&&(r=Math.cos(n),s=Math.sin(n));let o=this.center;Mr(yr.set(-.5,-.5,0),Ji,o,Ki,s,r),Mr(Ps.set(.5,-.5,0),Ji,o,Ki,s,r),Mr(vr.set(.5,.5,0),Ji,o,Ki,s,r),Gl.set(0,0),$o.set(1,0),Wl.set(1,1);let a=t.ray.intersectTriangle(yr,Ps,vr,!1,Cs);if(a===null&&(Mr(Ps.set(-.5,.5,0),Ji,o,Ki,s,r),$o.set(0,1),a=t.ray.intersectTriangle(yr,vr,Ps,!1,Cs),a===null))return;let c=t.ray.origin.distanceTo(Cs);c<t.near||c>t.far||e.push({distance:c,point:Cs.clone(),uv:ui.getInterpolation(Cs,yr,Ps,vr,Gl,$o,Wl,new Wt),face:null,object:this})}copy(t,e){return super.copy(t,e),t.center!==void 0&&this.center.copy(t.center),this.material=t.material,this}};function Mr(i,t,e,n,s,r){ji.subVectors(i,e).addScalar(.5).multiply(n),s!==void 0?(Rs.x=r*ji.x-s*ji.y,Rs.y=s*ji.x+r*ji.y):Rs.copy(ji),i.copy(t),i.x+=Rs.x,i.y+=Rs.y,i.applyMatrix4(xh)}var Ri=class extends nn{constructor(t=null,e=1,n=1,s,r,o,a,c,l=fn,h=fn,u,p){super(null,o,a,c,l,h,s,r,u,p),this.isDataTexture=!0,this.image={data:t,width:e,height:n},this.generateMipmaps=!1,this.flipY=!1,this.unpackAlignment=1}};var gi=class extends ve{constructor(t,e,n,s=1){super(t,e,n),this.isInstancedBufferAttribute=!0,this.meshPerAttribute=s}copy(t){return super.copy(t),this.meshPerAttribute=t.meshPerAttribute,this}toJSON(){let t=super.toJSON();return t.meshPerAttribute=this.meshPerAttribute,t.isInstancedBufferAttribute=!0,t}},Qi=new ce,Xl=new ce,Sr=[],Yl=new Qn,p0=new ce,Is=new se,Ls=new mi,An=class extends se{constructor(t,e,n){super(t,e),this.isInstancedMesh=!0,this.instanceMatrix=new gi(new Float32Array(n*16),16),this.instanceColor=null,this.morphTexture=null,this.count=n,this.boundingBox=null,this.boundingSphere=null;for(let s=0;s<n;s++)this.setMatrixAt(s,p0)}computeBoundingBox(){let t=this.geometry,e=this.count;this.boundingBox===null&&(this.boundingBox=new Qn),t.boundingBox===null&&t.computeBoundingBox(),this.boundingBox.makeEmpty();for(let n=0;n<e;n++)this.getMatrixAt(n,Qi),Yl.copy(t.boundingBox).applyMatrix4(Qi),this.boundingBox.union(Yl)}computeBoundingSphere(){let t=this.geometry,e=this.count;this.boundingSphere===null&&(this.boundingSphere=new mi),t.boundingSphere===null&&t.computeBoundingSphere(),this.boundingSphere.makeEmpty();for(let n=0;n<e;n++)this.getMatrixAt(n,Qi),Ls.copy(t.boundingSphere).applyMatrix4(Qi),this.boundingSphere.union(Ls)}copy(t,e){return super.copy(t,e),this.instanceMatrix.copy(t.instanceMatrix),t.morphTexture!==null&&(this.morphTexture=t.morphTexture.clone()),t.instanceColor!==null&&(this.instanceColor=t.instanceColor.clone()),this.count=t.count,t.boundingBox!==null&&(this.boundingBox=t.boundingBox.clone()),t.boundingSphere!==null&&(this.boundingSphere=t.boundingSphere.clone()),this}getColorAt(t,e){e.fromArray(this.instanceColor.array,t*3)}getMatrixAt(t,e){e.fromArray(this.instanceMatrix.array,t*16)}getMorphAt(t,e){let n=e.morphTargetInfluences,s=this.morphTexture.source.data.data,r=n.length+1,o=t*r+1;for(let a=0;a<n.length;a++)n[a]=s[o+a]}raycast(t,e){let n=this.matrixWorld,s=this.count;if(Is.geometry=this.geometry,Is.material=this.material,Is.material!==void 0&&(this.boundingSphere===null&&this.computeBoundingSphere(),Ls.copy(this.boundingSphere),Ls.applyMatrix4(n),t.ray.intersectsSphere(Ls)!==!1))for(let r=0;r<s;r++){this.getMatrixAt(r,Qi),Xl.multiplyMatrices(n,Qi),Is.matrixWorld=Xl,Is.raycast(t,Sr);for(let o=0,a=Sr.length;o<a;o++){let c=Sr[o];c.instanceId=r,c.object=this,e.push(c)}Sr.length=0}}setColorAt(t,e){this.instanceColor===null&&(this.instanceColor=new gi(new Float32Array(this.instanceMatrix.count*3).fill(1),3)),e.toArray(this.instanceColor.array,t*3)}setMatrixAt(t,e){e.toArray(this.instanceMatrix.array,t*16)}setMorphAt(t,e){let n=e.morphTargetInfluences,s=n.length+1;this.morphTexture===null&&(this.morphTexture=new Ri(new Float32Array(s*this.count),s,this.count,qs,_n));let r=this.morphTexture.source.data.data,o=0;for(let l=0;l<n.length;l++)o+=n[l];let a=this.geometry.morphTargetsRelative?1:1-o,c=s*t;r[c]=a,r.set(n,c+1)}updateMorphTargets(){}dispose(){return this.dispatchEvent({type:"dispose"}),this.morphTexture!==null&&(this.morphTexture.dispose(),this.morphTexture=null),this}};var Gs=class extends Nn{static get type(){return"PointsMaterial"}constructor(t){super(),this.isPointsMaterial=!0,this.color=new _t(16777215),this.map=null,this.alphaMap=null,this.size=1,this.sizeAttenuation=!0,this.fog=!0,this.setValues(t)}copy(t){return super.copy(t),this.color.copy(t.color),this.map=t.map,this.alphaMap=t.alphaMap,this.size=t.size,this.sizeAttenuation=t.sizeAttenuation,this.fog=t.fog,this}},ql=new ce,ja=new hs,br=new mi,Er=new U,Yr=class extends Re{constructor(t=new Ge,e=new Gs){super(),this.isPoints=!0,this.type="Points",this.geometry=t,this.material=e,this.updateMorphTargets()}copy(t,e){return super.copy(t,e),this.material=Array.isArray(t.material)?t.material.slice():t.material,this.geometry=t.geometry,this}raycast(t,e){let n=this.geometry,s=this.matrixWorld,r=t.params.Points.threshold,o=n.drawRange;if(n.boundingSphere===null&&n.computeBoundingSphere(),br.copy(n.boundingSphere),br.applyMatrix4(s),br.radius+=r,t.ray.intersectsSphere(br)===!1)return;ql.copy(s).invert(),ja.copy(t.ray).applyMatrix4(ql);let a=r/((this.scale.x+this.scale.y+this.scale.z)/3),c=a*a,l=n.index,u=n.attributes.position;if(l!==null){let p=Math.max(0,o.start),f=Math.min(l.count,o.start+o.count);for(let g=p,y=f;g<y;g++){let m=l.getX(g);Er.fromBufferAttribute(u,m),$l(Er,m,c,s,t,e,this)}}else{let p=Math.max(0,o.start),f=Math.min(u.count,o.start+o.count);for(let g=p,y=f;g<y;g++)Er.fromBufferAttribute(u,g),$l(Er,g,c,s,t,e,this)}}updateMorphTargets(){let e=this.geometry.morphAttributes,n=Object.keys(e);if(n.length>0){let s=e[n[0]];if(s!==void 0){this.morphTargetInfluences=[],this.morphTargetDictionary={};for(let r=0,o=s.length;r<o;r++){let a=s[r].name||String(r);this.morphTargetInfluences.push(0),this.morphTargetDictionary[a]=r}}}}};function $l(i,t,e,n,s,r,o){let a=ja.distanceSqToPoint(i);if(a<e){let c=new U;ja.closestPointToPoint(i,c),c.applyMatrix4(n);let l=s.ray.origin.distanceTo(c);if(l<s.near||l>s.far)return;r.push({distance:l,distanceToRay:Math.sqrt(a),point:c,index:t,face:null,faceIndex:null,barycoord:null,object:o})}}var Fn=class extends nn{constructor(t,e,n,s,r,o,a,c,l){super(t,e,n,s,r,o,a,c,l),this.isCanvasTexture=!0,this.needsUpdate=!0}};var De=class i extends Ge{constructor(t=1,e=1,n=1,s=32,r=1,o=!1,a=0,c=Math.PI*2){super(),this.type="CylinderGeometry",this.parameters={radiusTop:t,radiusBottom:e,height:n,radialSegments:s,heightSegments:r,openEnded:o,thetaStart:a,thetaLength:c};let l=this;s=Math.floor(s),r=Math.floor(r);let h=[],u=[],p=[],f=[],g=0,y=[],m=n/2,d=0;x(),o===!1&&(t>0&&_(!0),e>0&&_(!1)),this.setIndex(h),this.setAttribute("position",new Ce(u,3)),this.setAttribute("normal",new Ce(p,3)),this.setAttribute("uv",new Ce(f,2));function x(){let v=new U,C=new U,w=0,A=(e-t)/n;for(let P=0;P<=r;P++){let E=[],M=P/r,R=M*(e-t)+t;for(let F=0;F<=s;F++){let V=F/s,z=V*c+a,k=Math.sin(z),I=Math.cos(z);C.x=R*k,C.y=-M*n+m,C.z=R*I,u.push(C.x,C.y,C.z),v.set(k,A,I).normalize(),p.push(v.x,v.y,v.z),f.push(V,1-M),E.push(g++)}y.push(E)}for(let P=0;P<s;P++)for(let E=0;E<r;E++){let M=y[E][P],R=y[E+1][P],F=y[E+1][P+1],V=y[E][P+1];(t>0||E!==0)&&(h.push(M,R,V),w+=3),(e>0||E!==r-1)&&(h.push(R,F,V),w+=3)}l.addGroup(d,w,0),d+=w}function _(v){let C=g,w=new Wt,A=new U,P=0,E=v===!0?t:e,M=v===!0?1:-1;for(let F=1;F<=s;F++)u.push(0,m*M,0),p.push(0,M,0),f.push(.5,.5),g++;let R=g;for(let F=0;F<=s;F++){let z=F/s*c+a,k=Math.cos(z),I=Math.sin(z);A.x=E*I,A.y=m*M,A.z=E*k,u.push(A.x,A.y,A.z),p.push(0,M,0),w.x=k*.5+.5,w.y=I*.5*M+.5,f.push(w.x,w.y),g++}for(let F=0;F<s;F++){let V=C+F,z=R+F;v===!0?h.push(z,z+1,V):h.push(z+1,z,V),P+=3}l.addGroup(d,P,v===!0?1:2),d+=P}}copy(t){return super.copy(t),this.parameters=Object.assign({},t.parameters),this}static fromJSON(t){return new i(t.radiusTop,t.radiusBottom,t.height,t.radialSegments,t.heightSegments,t.openEnded,t.thetaStart,t.thetaLength)}},Cn=class i extends De{constructor(t=1,e=1,n=32,s=1,r=!1,o=0,a=Math.PI*2){super(0,t,e,n,s,r,o,a),this.type="ConeGeometry",this.parameters={radius:t,height:e,radialSegments:n,heightSegments:s,openEnded:r,thetaStart:o,thetaLength:a}}static fromJSON(t){return new i(t.radius,t.height,t.radialSegments,t.heightSegments,t.openEnded,t.thetaStart,t.thetaLength)}},qr=class i extends Ge{constructor(t=[],e=[],n=1,s=0){super(),this.type="PolyhedronGeometry",this.parameters={vertices:t,indices:e,radius:n,detail:s};let r=[],o=[];a(s),l(n),h(),this.setAttribute("position",new Ce(r,3)),this.setAttribute("normal",new Ce(r.slice(),3)),this.setAttribute("uv",new Ce(o,2)),s===0?this.computeVertexNormals():this.normalizeNormals();function a(x){let _=new U,v=new U,C=new U;for(let w=0;w<e.length;w+=3)f(e[w+0],_),f(e[w+1],v),f(e[w+2],C),c(_,v,C,x)}function c(x,_,v,C){let w=C+1,A=[];for(let P=0;P<=w;P++){A[P]=[];let E=x.clone().lerp(v,P/w),M=_.clone().lerp(v,P/w),R=w-P;for(let F=0;F<=R;F++)F===0&&P===w?A[P][F]=E:A[P][F]=E.clone().lerp(M,F/R)}for(let P=0;P<w;P++)for(let E=0;E<2*(w-P)-1;E++){let M=Math.floor(E/2);E%2===0?(p(A[P][M+1]),p(A[P+1][M]),p(A[P][M])):(p(A[P][M+1]),p(A[P+1][M+1]),p(A[P+1][M]))}}function l(x){let _=new U;for(let v=0;v<r.length;v+=3)_.x=r[v+0],_.y=r[v+1],_.z=r[v+2],_.normalize().multiplyScalar(x),r[v+0]=_.x,r[v+1]=_.y,r[v+2]=_.z}function h(){let x=new U;for(let _=0;_<r.length;_+=3){x.x=r[_+0],x.y=r[_+1],x.z=r[_+2];let v=m(x)/2/Math.PI+.5,C=d(x)/Math.PI+.5;o.push(v,1-C)}g(),u()}function u(){for(let x=0;x<o.length;x+=6){let _=o[x+0],v=o[x+2],C=o[x+4],w=Math.max(_,v,C),A=Math.min(_,v,C);w>.9&&A<.1&&(_<.2&&(o[x+0]+=1),v<.2&&(o[x+2]+=1),C<.2&&(o[x+4]+=1))}}function p(x){r.push(x.x,x.y,x.z)}function f(x,_){let v=x*3;_.x=t[v+0],_.y=t[v+1],_.z=t[v+2]}function g(){let x=new U,_=new U,v=new U,C=new U,w=new Wt,A=new Wt,P=new Wt;for(let E=0,M=0;E<r.length;E+=9,M+=6){x.set(r[E+0],r[E+1],r[E+2]),_.set(r[E+3],r[E+4],r[E+5]),v.set(r[E+6],r[E+7],r[E+8]),w.set(o[M+0],o[M+1]),A.set(o[M+2],o[M+3]),P.set(o[M+4],o[M+5]),C.copy(x).add(_).add(v).divideScalar(3);let R=m(C);y(w,M+0,x,R),y(A,M+2,_,R),y(P,M+4,v,R)}}function y(x,_,v,C){C<0&&x.x===1&&(o[_]=x.x-1),v.x===0&&v.z===0&&(o[_]=C/2/Math.PI+.5)}function m(x){return Math.atan2(x.z,-x.x)}function d(x){return Math.atan2(-x.y,Math.sqrt(x.x*x.x+x.z*x.z))}}copy(t){return super.copy(t),this.parameters=Object.assign({},t.parameters),this}static fromJSON(t){return new i(t.vertices,t.indices,t.radius,t.details)}};var $r=class i extends qr{constructor(t=1,e=0){let n=(1+Math.sqrt(5))/2,s=[-1,n,0,1,n,0,-1,-n,0,1,-n,0,0,-1,n,0,1,n,0,-1,-n,0,1,-n,n,0,-1,n,0,1,-n,0,-1,-n,0,1],r=[0,11,5,0,5,1,0,1,7,0,7,10,0,10,11,1,5,9,5,11,4,11,10,2,10,7,6,7,1,8,3,9,4,3,4,2,3,2,6,3,6,8,3,8,9,4,9,5,2,4,11,6,2,10,8,6,7,9,8,1];super(s,r,t,e),this.type="IcosahedronGeometry",this.parameters={radius:t,detail:e}}static fromJSON(t){return new i(t.radius,t.detail)}},Zr=class i extends qr{constructor(t=1,e=0){let n=[1,0,0,-1,0,0,0,1,0,0,-1,0,0,0,1,0,0,-1],s=[0,2,4,0,4,3,0,3,5,0,5,2,1,2,5,1,5,3,1,3,4,1,4,2];super(n,s,t,e),this.type="OctahedronGeometry",this.parameters={radius:t,detail:e}}static fromJSON(t){return new i(t.radius,t.detail)}};var $e=class i extends Ge{constructor(t=1,e=32,n=16,s=0,r=Math.PI*2,o=0,a=Math.PI){super(),this.type="SphereGeometry",this.parameters={radius:t,widthSegments:e,heightSegments:n,phiStart:s,phiLength:r,thetaStart:o,thetaLength:a},e=Math.max(3,Math.floor(e)),n=Math.max(2,Math.floor(n));let c=Math.min(o+a,Math.PI),l=0,h=[],u=new U,p=new U,f=[],g=[],y=[],m=[];for(let d=0;d<=n;d++){let x=[],_=d/n,v=0;d===0&&o===0?v=.5/e:d===n&&c===Math.PI&&(v=-.5/e);for(let C=0;C<=e;C++){let w=C/e;u.x=-t*Math.cos(s+w*r)*Math.sin(o+_*a),u.y=t*Math.cos(o+_*a),u.z=t*Math.sin(s+w*r)*Math.sin(o+_*a),g.push(u.x,u.y,u.z),p.copy(u).normalize(),y.push(p.x,p.y,p.z),m.push(w+v,1-_),x.push(l++)}h.push(x)}for(let d=0;d<n;d++)for(let x=0;x<e;x++){let _=h[d][x+1],v=h[d][x],C=h[d+1][x],w=h[d+1][x+1];(d!==0||o>0)&&f.push(_,v,w),(d!==n-1||c<Math.PI)&&f.push(v,C,w)}this.setIndex(f),this.setAttribute("position",new Ce(g,3)),this.setAttribute("normal",new Ce(y,3)),this.setAttribute("uv",new Ce(m,2))}copy(t){return super.copy(t),this.parameters=Object.assign({},t.parameters),this}static fromJSON(t){return new i(t.radius,t.widthSegments,t.heightSegments,t.phiStart,t.phiLength,t.thetaStart,t.thetaLength)}};var Ae=class extends Nn{static get type(){return"MeshStandardMaterial"}constructor(t){super(),this.isMeshStandardMaterial=!0,this.defines={STANDARD:""},this.color=new _t(16777215),this.roughness=1,this.metalness=0,this.map=null,this.lightMap=null,this.lightMapIntensity=1,this.aoMap=null,this.aoMapIntensity=1,this.emissive=new _t(0),this.emissiveIntensity=1,this.emissiveMap=null,this.bumpMap=null,this.bumpScale=1,this.normalMap=null,this.normalMapType=Mc,this.normalScale=new Wt(1,1),this.displacementMap=null,this.displacementScale=1,this.displacementBias=0,this.roughnessMap=null,this.metalnessMap=null,this.alphaMap=null,this.envMap=null,this.envMapRotation=new sn,this.envMapIntensity=1,this.wireframe=!1,this.wireframeLinewidth=1,this.wireframeLinecap="round",this.wireframeLinejoin="round",this.flatShading=!1,this.fog=!0,this.setValues(t)}copy(t){return super.copy(t),this.defines={STANDARD:""},this.color.copy(t.color),this.roughness=t.roughness,this.metalness=t.metalness,this.map=t.map,this.lightMap=t.lightMap,this.lightMapIntensity=t.lightMapIntensity,this.aoMap=t.aoMap,this.aoMapIntensity=t.aoMapIntensity,this.emissive.copy(t.emissive),this.emissiveMap=t.emissiveMap,this.emissiveIntensity=t.emissiveIntensity,this.bumpMap=t.bumpMap,this.bumpScale=t.bumpScale,this.normalMap=t.normalMap,this.normalMapType=t.normalMapType,this.normalScale.copy(t.normalScale),this.displacementMap=t.displacementMap,this.displacementScale=t.displacementScale,this.displacementBias=t.displacementBias,this.roughnessMap=t.roughnessMap,this.metalnessMap=t.metalnessMap,this.alphaMap=t.alphaMap,this.envMap=t.envMap,this.envMapRotation.copy(t.envMapRotation),this.envMapIntensity=t.envMapIntensity,this.wireframe=t.wireframe,this.wireframeLinewidth=t.wireframeLinewidth,this.wireframeLinecap=t.wireframeLinecap,this.wireframeLinejoin=t.wireframeLinejoin,this.flatShading=t.flatShading,this.fog=t.fog,this}};var Kr=class extends Nn{static get type(){return"MeshLambertMaterial"}constructor(t){super(),this.isMeshLambertMaterial=!0,this.color=new _t(16777215),this.map=null,this.lightMap=null,this.lightMapIntensity=1,this.aoMap=null,this.aoMapIntensity=1,this.emissive=new _t(0),this.emissiveIntensity=1,this.emissiveMap=null,this.bumpMap=null,this.bumpScale=1,this.normalMap=null,this.normalMapType=Mc,this.normalScale=new Wt(1,1),this.displacementMap=null,this.displacementScale=1,this.displacementBias=0,this.specularMap=null,this.alphaMap=null,this.envMap=null,this.envMapRotation=new sn,this.combine=fc,this.reflectivity=1,this.refractionRatio=.98,this.wireframe=!1,this.wireframeLinewidth=1,this.wireframeLinecap="round",this.wireframeLinejoin="round",this.flatShading=!1,this.fog=!0,this.setValues(t)}copy(t){return super.copy(t),this.color.copy(t.color),this.map=t.map,this.lightMap=t.lightMap,this.lightMapIntensity=t.lightMapIntensity,this.aoMap=t.aoMap,this.aoMapIntensity=t.aoMapIntensity,this.emissive.copy(t.emissive),this.emissiveMap=t.emissiveMap,this.emissiveIntensity=t.emissiveIntensity,this.bumpMap=t.bumpMap,this.bumpScale=t.bumpScale,this.normalMap=t.normalMap,this.normalMapType=t.normalMapType,this.normalScale.copy(t.normalScale),this.displacementMap=t.displacementMap,this.displacementScale=t.displacementScale,this.displacementBias=t.displacementBias,this.specularMap=t.specularMap,this.alphaMap=t.alphaMap,this.envMap=t.envMap,this.envMapRotation.copy(t.envMapRotation),this.combine=t.combine,this.reflectivity=t.reflectivity,this.refractionRatio=t.refractionRatio,this.wireframe=t.wireframe,this.wireframeLinewidth=t.wireframeLinewidth,this.wireframeLinecap=t.wireframeLinecap,this.wireframeLinejoin=t.wireframeLinejoin,this.flatShading=t.flatShading,this.fog=t.fog,this}};function Tr(i,t,e){return!i||!e&&i.constructor===t?i:typeof t.BYTES_PER_ELEMENT=="number"?new t(i):Array.prototype.slice.call(i)}function m0(i){return ArrayBuffer.isView(i)&&!(i instanceof DataView)}var ms=class{constructor(t,e,n,s){this.parameterPositions=t,this._cachedIndex=0,this.resultBuffer=s!==void 0?s:new e.constructor(n),this.sampleValues=e,this.valueSize=n,this.settings=null,this.DefaultSettings_={}}evaluate(t){let e=this.parameterPositions,n=this._cachedIndex,s=e[n],r=e[n-1];n:{t:{let o;e:{i:if(!(t<s)){for(let a=n+2;;){if(s===void 0){if(t<r)break i;return n=e.length,this._cachedIndex=n,this.copySampleValue_(n-1)}if(n===a)break;if(r=s,s=e[++n],t<s)break t}o=e.length;break e}if(!(t>=r)){let a=e[1];t<a&&(n=2,r=a);for(let c=n-2;;){if(r===void 0)return this._cachedIndex=0,this.copySampleValue_(0);if(n===c)break;if(s=r,r=e[--n-1],t>=r)break t}o=n,n=0;break e}break n}for(;n<o;){let a=n+o>>>1;t<e[a]?o=a:n=a+1}if(s=e[n],r=e[n-1],r===void 0)return this._cachedIndex=0,this.copySampleValue_(0);if(s===void 0)return n=e.length,this._cachedIndex=n,this.copySampleValue_(n-1)}this._cachedIndex=n,this.intervalChanged_(n,r,s)}return this.interpolate_(n,r,t,s)}getSettings_(){return this.settings||this.DefaultSettings_}copySampleValue_(t){let e=this.resultBuffer,n=this.sampleValues,s=this.valueSize,r=t*s;for(let o=0;o!==s;++o)e[o]=n[r+o];return e}interpolate_(){throw new Error("call to abstract method")}intervalChanged_(){}},Qa=class extends ms{constructor(t,e,n,s){super(t,e,n,s),this._weightPrev=-0,this._offsetPrev=-0,this._weightNext=-0,this._offsetNext=-0,this.DefaultSettings_={endingStart:qc,endingEnd:qc}}intervalChanged_(t,e,n){let s=this.parameterPositions,r=t-2,o=t+1,a=s[r],c=s[o];if(a===void 0)switch(this.getSettings_().endingStart){case $c:r=t,a=2*e-n;break;case Zc:r=s.length-2,a=e+s[r]-s[r+1];break;default:r=t,a=n}if(c===void 0)switch(this.getSettings_().endingEnd){case $c:o=t,c=2*n-e;break;case Zc:o=1,c=n+s[1]-s[0];break;default:o=t-1,c=e}let l=(n-e)*.5,h=this.valueSize;this._weightPrev=l/(e-a),this._weightNext=l/(c-n),this._offsetPrev=r*h,this._offsetNext=o*h}interpolate_(t,e,n,s){let r=this.resultBuffer,o=this.sampleValues,a=this.valueSize,c=t*a,l=c-a,h=this._offsetPrev,u=this._offsetNext,p=this._weightPrev,f=this._weightNext,g=(n-e)/(s-e),y=g*g,m=y*g,d=-p*m+2*p*y-p*g,x=(1+p)*m+(-1.5-2*p)*y+(-.5+p)*g+1,_=(-1-f)*m+(1.5+f)*y+.5*g,v=f*m-f*y;for(let C=0;C!==a;++C)r[C]=d*o[h+C]+x*o[l+C]+_*o[c+C]+v*o[u+C];return r}},tc=class extends ms{constructor(t,e,n,s){super(t,e,n,s)}interpolate_(t,e,n,s){let r=this.resultBuffer,o=this.sampleValues,a=this.valueSize,c=t*a,l=c-a,h=(n-e)/(s-e),u=1-h;for(let p=0;p!==a;++p)r[p]=o[l+p]*u+o[c+p]*h;return r}},ec=class extends ms{constructor(t,e,n,s){super(t,e,n,s)}interpolate_(t){return this.copySampleValue_(t-1)}},Rn=class{constructor(t,e,n,s){if(t===void 0)throw new Error("THREE.KeyframeTrack: track name is undefined");if(e===void 0||e.length===0)throw new Error("THREE.KeyframeTrack: no keyframes in track named "+t);this.name=t,this.times=Tr(e,this.TimeBufferType),this.values=Tr(n,this.ValueBufferType),this.setInterpolation(s||this.DefaultInterpolation)}static toJSON(t){let e=t.constructor,n;if(e.toJSON!==this.toJSON)n=e.toJSON(t);else{n={name:t.name,times:Tr(t.times,Array),values:Tr(t.values,Array)};let s=t.getInterpolation();s!==t.DefaultInterpolation&&(n.interpolation=s)}return n.type=t.ValueTypeName,n}InterpolantFactoryMethodDiscrete(t){return new ec(this.times,this.values,this.getValueSize(),t)}InterpolantFactoryMethodLinear(t){return new tc(this.times,this.values,this.getValueSize(),t)}InterpolantFactoryMethodSmooth(t){return new Qa(this.times,this.values,this.getValueSize(),t)}setInterpolation(t){let e;switch(t){case Ir:e=this.InterpolantFactoryMethodDiscrete;break;case Ua:e=this.InterpolantFactoryMethodLinear;break;case Mo:e=this.InterpolantFactoryMethodSmooth;break}if(e===void 0){let n="unsupported interpolation for "+this.ValueTypeName+" keyframe track named "+this.name;if(this.createInterpolant===void 0)if(t!==this.DefaultInterpolation)this.setInterpolation(this.DefaultInterpolation);else throw new Error(n);return console.warn("THREE.KeyframeTrack:",n),this}return this.createInterpolant=e,this}getInterpolation(){switch(this.createInterpolant){case this.InterpolantFactoryMethodDiscrete:return Ir;case this.InterpolantFactoryMethodLinear:return Ua;case this.InterpolantFactoryMethodSmooth:return Mo}}getValueSize(){return this.values.length/this.times.length}shift(t){if(t!==0){let e=this.times;for(let n=0,s=e.length;n!==s;++n)e[n]+=t}return this}scale(t){if(t!==1){let e=this.times;for(let n=0,s=e.length;n!==s;++n)e[n]*=t}return this}trim(t,e){let n=this.times,s=n.length,r=0,o=s-1;for(;r!==s&&n[r]<t;)++r;for(;o!==-1&&n[o]>e;)--o;if(++o,r!==0||o!==s){r>=o&&(o=Math.max(o,1),r=o-1);let a=this.getValueSize();this.times=n.slice(r,o),this.values=this.values.slice(r*a,o*a)}return this}validate(){let t=!0,e=this.getValueSize();e-Math.floor(e)!==0&&(console.error("THREE.KeyframeTrack: Invalid value size in track.",this),t=!1);let n=this.times,s=this.values,r=n.length;r===0&&(console.error("THREE.KeyframeTrack: Track is empty.",this),t=!1);let o=null;for(let a=0;a!==r;a++){let c=n[a];if(typeof c=="number"&&isNaN(c)){console.error("THREE.KeyframeTrack: Time is not a valid number.",this,a,c),t=!1;break}if(o!==null&&o>c){console.error("THREE.KeyframeTrack: Out of order keys.",this,a,c,o),t=!1;break}o=c}if(s!==void 0&&m0(s))for(let a=0,c=s.length;a!==c;++a){let l=s[a];if(isNaN(l)){console.error("THREE.KeyframeTrack: Value is not a valid number.",this,a,l),t=!1;break}}return t}optimize(){let t=this.times.slice(),e=this.values.slice(),n=this.getValueSize(),s=this.getInterpolation()===Mo,r=t.length-1,o=1;for(let a=1;a<r;++a){let c=!1,l=t[a],h=t[a+1];if(l!==h&&(a!==1||l!==t[0]))if(s)c=!0;else{let u=a*n,p=u-n,f=u+n;for(let g=0;g!==n;++g){let y=e[u+g];if(y!==e[p+g]||y!==e[f+g]){c=!0;break}}}if(c){if(a!==o){t[o]=t[a];let u=a*n,p=o*n;for(let f=0;f!==n;++f)e[p+f]=e[u+f]}++o}}if(r>0){t[o]=t[r];for(let a=r*n,c=o*n,l=0;l!==n;++l)e[c+l]=e[a+l];++o}return o!==t.length?(this.times=t.slice(0,o),this.values=e.slice(0,o*n)):(this.times=t,this.values=e),this}clone(){let t=this.times.slice(),e=this.values.slice(),n=this.constructor,s=new n(this.name,t,e);return s.createInterpolant=this.createInterpolant,s}};Rn.prototype.TimeBufferType=Float32Array;Rn.prototype.ValueBufferType=Float32Array;Rn.prototype.DefaultInterpolation=Ua;var Pi=class extends Rn{constructor(t,e,n){super(t,e,n)}};Pi.prototype.ValueTypeName="bool";Pi.prototype.ValueBufferType=Array;Pi.prototype.DefaultInterpolation=Ir;Pi.prototype.InterpolantFactoryMethodLinear=void 0;Pi.prototype.InterpolantFactoryMethodSmooth=void 0;var nc=class extends Rn{};nc.prototype.ValueTypeName="color";var ic=class extends Rn{};ic.prototype.ValueTypeName="number";var sc=class extends ms{constructor(t,e,n,s){super(t,e,n,s)}interpolate_(t,e,n,s){let r=this.resultBuffer,o=this.sampleValues,a=this.valueSize,c=(n-e)/(s-e),l=t*a;for(let h=l+a;l!==h;l+=4)qe.slerpFlat(r,0,o,l-a,o,l,c);return r}},Jr=class extends Rn{InterpolantFactoryMethodLinear(t){return new sc(this.times,this.values,this.getValueSize(),t)}};Jr.prototype.ValueTypeName="quaternion";Jr.prototype.InterpolantFactoryMethodSmooth=void 0;var Ii=class extends Rn{constructor(t,e,n){super(t,e,n)}};Ii.prototype.ValueTypeName="string";Ii.prototype.ValueBufferType=Array;Ii.prototype.DefaultInterpolation=Ir;Ii.prototype.InterpolantFactoryMethodLinear=void 0;Ii.prototype.InterpolantFactoryMethodSmooth=void 0;var rc=class extends Rn{};rc.prototype.ValueTypeName="vector";var oc=class{constructor(t,e,n){let s=this,r=!1,o=0,a=0,c,l=[];this.onStart=void 0,this.onLoad=t,this.onProgress=e,this.onError=n,this.itemStart=function(h){a++,r===!1&&s.onStart!==void 0&&s.onStart(h,o,a),r=!0},this.itemEnd=function(h){o++,s.onProgress!==void 0&&s.onProgress(h,o,a),o===a&&(r=!1,s.onLoad!==void 0&&s.onLoad())},this.itemError=function(h){s.onError!==void 0&&s.onError(h)},this.resolveURL=function(h){return c?c(h):h},this.setURLModifier=function(h){return c=h,this},this.addHandler=function(h,u){return l.push(h,u),this},this.removeHandler=function(h){let u=l.indexOf(h);return u!==-1&&l.splice(u,2),this},this.getHandler=function(h){for(let u=0,p=l.length;u<p;u+=2){let f=l[u],g=l[u+1];if(f.global&&(f.lastIndex=0),f.test(h))return g}return null}}},g0=new oc,ac=class{constructor(t){this.manager=t!==void 0?t:g0,this.crossOrigin="anonymous",this.withCredentials=!1,this.path="",this.resourcePath="",this.requestHeader={}}load(){}loadAsync(t,e){let n=this;return new Promise(function(s,r){n.load(t,s,e,r)})}parse(){}setCrossOrigin(t){return this.crossOrigin=t,this}setWithCredentials(t){return this.withCredentials=t,this}setPath(t){return this.path=t,this}setResourcePath(t){return this.resourcePath=t,this}setRequestHeader(t){return this.requestHeader=t,this}};ac.DEFAULT_MATERIAL_NAME="__DEFAULT";var Ws=class extends Re{constructor(t,e=1){super(),this.isLight=!0,this.type="Light",this.color=new _t(t),this.intensity=e}dispose(){}copy(t,e){return super.copy(t,e),this.color.copy(t.color),this.intensity=t.intensity,this}toJSON(t){let e=super.toJSON(t);return e.object.color=this.color.getHex(),e.object.intensity=this.intensity,this.groundColor!==void 0&&(e.object.groundColor=this.groundColor.getHex()),this.distance!==void 0&&(e.object.distance=this.distance),this.angle!==void 0&&(e.object.angle=this.angle),this.decay!==void 0&&(e.object.decay=this.decay),this.penumbra!==void 0&&(e.object.penumbra=this.penumbra),this.shadow!==void 0&&(e.object.shadow=this.shadow.toJSON()),this.target!==void 0&&(e.object.target=this.target.uuid),e}},jr=class extends Ws{constructor(t,e,n){super(t,n),this.isHemisphereLight=!0,this.type="HemisphereLight",this.position.copy(Re.DEFAULT_UP),this.updateMatrix(),this.groundColor=new _t(e)}copy(t,e){return super.copy(t,e),this.groundColor.copy(t.groundColor),this}},Zo=new ce,Zl=new U,Kl=new U,cc=class{constructor(t){this.camera=t,this.intensity=1,this.bias=0,this.normalBias=0,this.radius=1,this.blurSamples=8,this.mapSize=new Wt(512,512),this.map=null,this.mapPass=null,this.matrix=new ce,this.autoUpdate=!0,this.needsUpdate=!1,this._frustum=new ks,this._frameExtents=new Wt(1,1),this._viewportCount=1,this._viewports=[new be(0,0,1,1)]}getViewportCount(){return this._viewportCount}getFrustum(){return this._frustum}updateMatrices(t){let e=this.camera,n=this.matrix;Zl.setFromMatrixPosition(t.matrixWorld),e.position.copy(Zl),Kl.setFromMatrixPosition(t.target.matrixWorld),e.lookAt(Kl),e.updateMatrixWorld(),Zo.multiplyMatrices(e.projectionMatrix,e.matrixWorldInverse),this._frustum.setFromProjectionMatrix(Zo),n.set(.5,0,0,.5,0,.5,0,.5,0,0,.5,.5,0,0,0,1),n.multiply(Zo)}getViewport(t){return this._viewports[t]}getFrameExtents(){return this._frameExtents}dispose(){this.map&&this.map.dispose(),this.mapPass&&this.mapPass.dispose()}copy(t){return this.camera=t.camera.clone(),this.intensity=t.intensity,this.bias=t.bias,this.radius=t.radius,this.mapSize.copy(t.mapSize),this}clone(){return new this.constructor().copy(this)}toJSON(){let t={};return this.intensity!==1&&(t.intensity=this.intensity),this.bias!==0&&(t.bias=this.bias),this.normalBias!==0&&(t.normalBias=this.normalBias),this.radius!==1&&(t.radius=this.radius),(this.mapSize.x!==512||this.mapSize.y!==512)&&(t.mapSize=this.mapSize.toArray()),t.camera=this.camera.toJSON(!1).object,delete t.camera.matrix,t}};var lc=class extends cc{constructor(){super(new Hr(-5,5,5,-5,.5,500)),this.isDirectionalLightShadow=!0}},gs=class extends Ws{constructor(t,e){super(t,e),this.isDirectionalLight=!0,this.type="DirectionalLight",this.position.copy(Re.DEFAULT_UP),this.updateMatrix(),this.target=new Re,this.shadow=new lc}dispose(){this.shadow.dispose()}copy(t){return super.copy(t),this.target=t.target.clone(),this.shadow=t.shadow.clone(),this}},Qr=class extends Ws{constructor(t,e){super(t,e),this.isAmbientLight=!0,this.type="AmbientLight"}};var to=class{constructor(t=!0){this.autoStart=t,this.startTime=0,this.oldTime=0,this.elapsedTime=0,this.running=!1}start(){this.startTime=Jl(),this.oldTime=this.startTime,this.elapsedTime=0,this.running=!0}stop(){this.getElapsedTime(),this.running=!1,this.autoStart=!1}getElapsedTime(){return this.getDelta(),this.elapsedTime}getDelta(){let t=0;if(this.autoStart&&!this.running)return this.start(),0;if(this.running){let e=Jl();t=(e-this.oldTime)/1e3,this.oldTime=e,this.elapsedTime+=t}return t}};function Jl(){return performance.now()}var Ec="\\[\\]\\.:\\/",_0=new RegExp("["+Ec+"]","g"),Tc="[^"+Ec+"]",x0="[^"+Ec.replace("\\.","")+"]",y0=/((?:WC+[\/:])*)/.source.replace("WC",Tc),v0=/(WCOD+)?/.source.replace("WCOD",x0),M0=/(?:\.(WC+)(?:\[(.+)\])?)?/.source.replace("WC",Tc),S0=/\.(WC+)(?:\[(.+)\])?/.source.replace("WC",Tc),b0=new RegExp("^"+y0+v0+M0+S0+"$"),E0=["material","materials","bones","map"],hc=class{constructor(t,e,n){let s=n||Ee.parseTrackName(e);this._targetGroup=t,this._bindings=t.subscribe_(e,s)}getValue(t,e){this.bind();let n=this._targetGroup.nCachedObjects_,s=this._bindings[n];s!==void 0&&s.getValue(t,e)}setValue(t,e){let n=this._bindings;for(let s=this._targetGroup.nCachedObjects_,r=n.length;s!==r;++s)n[s].setValue(t,e)}bind(){let t=this._bindings;for(let e=this._targetGroup.nCachedObjects_,n=t.length;e!==n;++e)t[e].bind()}unbind(){let t=this._bindings;for(let e=this._targetGroup.nCachedObjects_,n=t.length;e!==n;++e)t[e].unbind()}},Ee=class i{constructor(t,e,n){this.path=e,this.parsedPath=n||i.parseTrackName(e),this.node=i.findNode(t,this.parsedPath.nodeName),this.rootNode=t,this.getValue=this._getValue_unbound,this.setValue=this._setValue_unbound}static create(t,e,n){return t&&t.isAnimationObjectGroup?new i.Composite(t,e,n):new i(t,e,n)}static sanitizeNodeName(t){return t.replace(/\s/g,"_").replace(_0,"")}static parseTrackName(t){let e=b0.exec(t);if(e===null)throw new Error("PropertyBinding: Cannot parse trackName: "+t);let n={nodeName:e[2],objectName:e[3],objectIndex:e[4],propertyName:e[5],propertyIndex:e[6]},s=n.nodeName&&n.nodeName.lastIndexOf(".");if(s!==void 0&&s!==-1){let r=n.nodeName.substring(s+1);E0.indexOf(r)!==-1&&(n.nodeName=n.nodeName.substring(0,s),n.objectName=r)}if(n.propertyName===null||n.propertyName.length===0)throw new Error("PropertyBinding: can not parse propertyName from trackName: "+t);return n}static findNode(t,e){if(e===void 0||e===""||e==="."||e===-1||e===t.name||e===t.uuid)return t;if(t.skeleton){let n=t.skeleton.getBoneByName(e);if(n!==void 0)return n}if(t.children){let n=function(r){for(let o=0;o<r.length;o++){let a=r[o];if(a.name===e||a.uuid===e)return a;let c=n(a.children);if(c)return c}return null},s=n(t.children);if(s)return s}return null}_getValue_unavailable(){}_setValue_unavailable(){}_getValue_direct(t,e){t[e]=this.targetObject[this.propertyName]}_getValue_array(t,e){let n=this.resolvedProperty;for(let s=0,r=n.length;s!==r;++s)t[e++]=n[s]}_getValue_arrayElement(t,e){t[e]=this.resolvedProperty[this.propertyIndex]}_getValue_toArray(t,e){this.resolvedProperty.toArray(t,e)}_setValue_direct(t,e){this.targetObject[this.propertyName]=t[e]}_setValue_direct_setNeedsUpdate(t,e){this.targetObject[this.propertyName]=t[e],this.targetObject.needsUpdate=!0}_setValue_direct_setMatrixWorldNeedsUpdate(t,e){this.targetObject[this.propertyName]=t[e],this.targetObject.matrixWorldNeedsUpdate=!0}_setValue_array(t,e){let n=this.resolvedProperty;for(let s=0,r=n.length;s!==r;++s)n[s]=t[e++]}_setValue_array_setNeedsUpdate(t,e){let n=this.resolvedProperty;for(let s=0,r=n.length;s!==r;++s)n[s]=t[e++];this.targetObject.needsUpdate=!0}_setValue_array_setMatrixWorldNeedsUpdate(t,e){let n=this.resolvedProperty;for(let s=0,r=n.length;s!==r;++s)n[s]=t[e++];this.targetObject.matrixWorldNeedsUpdate=!0}_setValue_arrayElement(t,e){this.resolvedProperty[this.propertyIndex]=t[e]}_setValue_arrayElement_setNeedsUpdate(t,e){this.resolvedProperty[this.propertyIndex]=t[e],this.targetObject.needsUpdate=!0}_setValue_arrayElement_setMatrixWorldNeedsUpdate(t,e){this.resolvedProperty[this.propertyIndex]=t[e],this.targetObject.matrixWorldNeedsUpdate=!0}_setValue_fromArray(t,e){this.resolvedProperty.fromArray(t,e)}_setValue_fromArray_setNeedsUpdate(t,e){this.resolvedProperty.fromArray(t,e),this.targetObject.needsUpdate=!0}_setValue_fromArray_setMatrixWorldNeedsUpdate(t,e){this.resolvedProperty.fromArray(t,e),this.targetObject.matrixWorldNeedsUpdate=!0}_getValue_unbound(t,e){this.bind(),this.getValue(t,e)}_setValue_unbound(t,e){this.bind(),this.setValue(t,e)}bind(){let t=this.node,e=this.parsedPath,n=e.objectName,s=e.propertyName,r=e.propertyIndex;if(t||(t=i.findNode(this.rootNode,e.nodeName),this.node=t),this.getValue=this._getValue_unavailable,this.setValue=this._setValue_unavailable,!t){console.warn("THREE.PropertyBinding: No target node found for track: "+this.path+".");return}if(n){let l=e.objectIndex;switch(n){case"materials":if(!t.material){console.error("THREE.PropertyBinding: Can not bind to material as node does not have a material.",this);return}if(!t.material.materials){console.error("THREE.PropertyBinding: Can not bind to material.materials as node.material does not have a materials array.",this);return}t=t.material.materials;break;case"bones":if(!t.skeleton){console.error("THREE.PropertyBinding: Can not bind to bones as node does not have a skeleton.",this);return}t=t.skeleton.bones;for(let h=0;h<t.length;h++)if(t[h].name===l){l=h;break}break;case"map":if("map"in t){t=t.map;break}if(!t.material){console.error("THREE.PropertyBinding: Can not bind to material as node does not have a material.",this);return}if(!t.material.map){console.error("THREE.PropertyBinding: Can not bind to material.map as node.material does not have a map.",this);return}t=t.material.map;break;default:if(t[n]===void 0){console.error("THREE.PropertyBinding: Can not bind to objectName of node undefined.",this);return}t=t[n]}if(l!==void 0){if(t[l]===void 0){console.error("THREE.PropertyBinding: Trying to bind to objectIndex of objectName, but is undefined.",this,t);return}t=t[l]}}let o=t[s];if(o===void 0){let l=e.nodeName;console.error("THREE.PropertyBinding: Trying to update property for track: "+l+"."+s+" but it wasn't found.",t);return}let a=this.Versioning.None;this.targetObject=t,t.needsUpdate!==void 0?a=this.Versioning.NeedsUpdate:t.matrixWorldNeedsUpdate!==void 0&&(a=this.Versioning.MatrixWorldNeedsUpdate);let c=this.BindingType.Direct;if(r!==void 0){if(s==="morphTargetInfluences"){if(!t.geometry){console.error("THREE.PropertyBinding: Can not bind to morphTargetInfluences because node does not have a geometry.",this);return}if(!t.geometry.morphAttributes){console.error("THREE.PropertyBinding: Can not bind to morphTargetInfluences because node does not have a geometry.morphAttributes.",this);return}t.morphTargetDictionary[r]!==void 0&&(r=t.morphTargetDictionary[r])}c=this.BindingType.ArrayElement,this.resolvedProperty=o,this.propertyIndex=r}else o.fromArray!==void 0&&o.toArray!==void 0?(c=this.BindingType.HasFromToArray,this.resolvedProperty=o):Array.isArray(o)?(c=this.BindingType.EntireArray,this.resolvedProperty=o):this.propertyName=s;this.getValue=this.GetterByBindingType[c],this.setValue=this.SetterByBindingTypeAndVersioning[c][a]}unbind(){this.node=null,this.getValue=this._getValue_unbound,this.setValue=this._setValue_unbound}};Ee.Composite=hc;Ee.prototype.BindingType={Direct:0,EntireArray:1,ArrayElement:2,HasFromToArray:3};Ee.prototype.Versioning={None:0,NeedsUpdate:1,MatrixWorldNeedsUpdate:2};Ee.prototype.GetterByBindingType=[Ee.prototype._getValue_direct,Ee.prototype._getValue_array,Ee.prototype._getValue_arrayElement,Ee.prototype._getValue_toArray];Ee.prototype.SetterByBindingTypeAndVersioning=[[Ee.prototype._setValue_direct,Ee.prototype._setValue_direct_setNeedsUpdate,Ee.prototype._setValue_direct_setMatrixWorldNeedsUpdate],[Ee.prototype._setValue_array,Ee.prototype._setValue_array_setNeedsUpdate,Ee.prototype._setValue_array_setMatrixWorldNeedsUpdate],[Ee.prototype._setValue_arrayElement,Ee.prototype._setValue_arrayElement_setNeedsUpdate,Ee.prototype._setValue_arrayElement_setMatrixWorldNeedsUpdate],[Ee.prototype._setValue_fromArray,Ee.prototype._setValue_fromArray_setNeedsUpdate,Ee.prototype._setValue_fromArray_setMatrixWorldNeedsUpdate]];var K_=new Float32Array(1);var Xs=class{constructor(t=1,e=0,n=0){return this.radius=t,this.phi=e,this.theta=n,this}set(t,e,n){return this.radius=t,this.phi=e,this.theta=n,this}copy(t){return this.radius=t.radius,this.phi=t.phi,this.theta=t.theta,this}makeSafe(){return this.phi=Math.max(1e-6,Math.min(Math.PI-1e-6,this.phi)),this}setFromVector3(t){return this.setFromCartesianCoords(t.x,t.y,t.z)}setFromCartesianCoords(t,e,n){return this.radius=Math.sqrt(t*t+e*e+n*n),this.radius===0?(this.theta=0,this.phi=0):(this.theta=Math.atan2(t,n),this.phi=Math.acos(Ye(e/this.radius,-1,1))),this}clone(){return new this.constructor().copy(this)}};var eo=class extends jn{constructor(t,e=null){super(),this.object=t,this.domElement=e,this.enabled=!0,this.state=-1,this.keys={},this.mouseButtons={LEFT:null,MIDDLE:null,RIGHT:null},this.touches={ONE:null,TWO:null}}connect(){}disconnect(){}dispose(){}update(){}};typeof __THREE_DEVTOOLS__<"u"&&__THREE_DEVTOOLS__.dispatchEvent(new CustomEvent("register",{detail:{revision:uc}}));typeof window<"u"&&(window.__THREE__?console.warn("WARNING: Multiple instances of Three.js being imported."):window.__THREE__=uc);var yh={type:"change"},Ac={type:"start"},Mh={type:"end"},oo=new hs,vh=new un,T0=Math.cos(70*$s.DEG2RAD),Ue=new U,rn=2*Math.PI,ye={NONE:-1,ROTATE:0,DOLLY:1,PAN:2,TOUCH_ROTATE:3,TOUCH_PAN:4,TOUCH_DOLLY_PAN:5,TOUCH_DOLLY_ROTATE:6},wc=1e-6,ao=class extends eo{constructor(t,e=null){super(t,e),this.state=ye.NONE,this.enabled=!0,this.target=new U,this.cursor=new U,this.minDistance=0,this.maxDistance=1/0,this.minZoom=0,this.maxZoom=1/0,this.minTargetRadius=0,this.maxTargetRadius=1/0,this.minPolarAngle=0,this.maxPolarAngle=Math.PI,this.minAzimuthAngle=-1/0,this.maxAzimuthAngle=1/0,this.enableDamping=!1,this.dampingFactor=.05,this.enableZoom=!0,this.zoomSpeed=1,this.enableRotate=!0,this.rotateSpeed=1,this.enablePan=!0,this.panSpeed=1,this.screenSpacePanning=!0,this.keyPanSpeed=7,this.zoomToCursor=!1,this.autoRotate=!1,this.autoRotateSpeed=2,this.keys={LEFT:"ArrowLeft",UP:"ArrowUp",RIGHT:"ArrowRight",BOTTOM:"ArrowDown"},this.mouseButtons={LEFT:Li.ROTATE,MIDDLE:Li.DOLLY,RIGHT:Li.PAN},this.touches={ONE:Di.ROTATE,TWO:Di.DOLLY_PAN},this.target0=this.target.clone(),this.position0=this.object.position.clone(),this.zoom0=this.object.zoom,this._domElementKeyEvents=null,this._lastPosition=new U,this._lastQuaternion=new qe,this._lastTargetPosition=new U,this._quat=new qe().setFromUnitVectors(t.up,new U(0,1,0)),this._quatInverse=this._quat.clone().invert(),this._spherical=new Xs,this._sphericalDelta=new Xs,this._scale=1,this._panOffset=new U,this._rotateStart=new Wt,this._rotateEnd=new Wt,this._rotateDelta=new Wt,this._panStart=new Wt,this._panEnd=new Wt,this._panDelta=new Wt,this._dollyStart=new Wt,this._dollyEnd=new Wt,this._dollyDelta=new Wt,this._dollyDirection=new U,this._mouse=new Wt,this._performCursorZoom=!1,this._pointers=[],this._pointerPositions={},this._controlActive=!1,this._onPointerMove=A0.bind(this),this._onPointerDown=w0.bind(this),this._onPointerUp=C0.bind(this),this._onContextMenu=N0.bind(this),this._onMouseWheel=I0.bind(this),this._onKeyDown=L0.bind(this),this._onTouchStart=D0.bind(this),this._onTouchMove=U0.bind(this),this._onMouseDown=R0.bind(this),this._onMouseMove=P0.bind(this),this._interceptControlDown=F0.bind(this),this._interceptControlUp=O0.bind(this),this.domElement!==null&&this.connect(),this.update()}connect(){this.domElement.addEventListener("pointerdown",this._onPointerDown),this.domElement.addEventListener("pointercancel",this._onPointerUp),this.domElement.addEventListener("contextmenu",this._onContextMenu),this.domElement.addEventListener("wheel",this._onMouseWheel,{passive:!1}),this.domElement.getRootNode().addEventListener("keydown",this._interceptControlDown,{passive:!0,capture:!0}),this.domElement.style.touchAction="none"}disconnect(){this.domElement.removeEventListener("pointerdown",this._onPointerDown),this.domElement.removeEventListener("pointermove",this._onPointerMove),this.domElement.removeEventListener("pointerup",this._onPointerUp),this.domElement.removeEventListener("pointercancel",this._onPointerUp),this.domElement.removeEventListener("wheel",this._onMouseWheel),this.domElement.removeEventListener("contextmenu",this._onContextMenu),this.stopListenToKeyEvents(),this.domElement.getRootNode().removeEventListener("keydown",this._interceptControlDown,{capture:!0}),this.domElement.style.touchAction="auto"}dispose(){this.disconnect()}getPolarAngle(){return this._spherical.phi}getAzimuthalAngle(){return this._spherical.theta}getDistance(){return this.object.position.distanceTo(this.target)}listenToKeyEvents(t){t.addEventListener("keydown",this._onKeyDown),this._domElementKeyEvents=t}stopListenToKeyEvents(){this._domElementKeyEvents!==null&&(this._domElementKeyEvents.removeEventListener("keydown",this._onKeyDown),this._domElementKeyEvents=null)}saveState(){this.target0.copy(this.target),this.position0.copy(this.object.position),this.zoom0=this.object.zoom}reset(){this.target.copy(this.target0),this.object.position.copy(this.position0),this.object.zoom=this.zoom0,this.object.updateProjectionMatrix(),this.dispatchEvent(yh),this.update(),this.state=ye.NONE}update(t=null){let e=this.object.position;Ue.copy(e).sub(this.target),Ue.applyQuaternion(this._quat),this._spherical.setFromVector3(Ue),this.autoRotate&&this.state===ye.NONE&&this._rotateLeft(this._getAutoRotationAngle(t)),this.enableDamping?(this._spherical.theta+=this._sphericalDelta.theta*this.dampingFactor,this._spherical.phi+=this._sphericalDelta.phi*this.dampingFactor):(this._spherical.theta+=this._sphericalDelta.theta,this._spherical.phi+=this._sphericalDelta.phi);let n=this.minAzimuthAngle,s=this.maxAzimuthAngle;isFinite(n)&&isFinite(s)&&(n<-Math.PI?n+=rn:n>Math.PI&&(n-=rn),s<-Math.PI?s+=rn:s>Math.PI&&(s-=rn),n<=s?this._spherical.theta=Math.max(n,Math.min(s,this._spherical.theta)):this._spherical.theta=this._spherical.theta>(n+s)/2?Math.max(n,this._spherical.theta):Math.min(s,this._spherical.theta)),this._spherical.phi=Math.max(this.minPolarAngle,Math.min(this.maxPolarAngle,this._spherical.phi)),this._spherical.makeSafe(),this.enableDamping===!0?this.target.addScaledVector(this._panOffset,this.dampingFactor):this.target.add(this._panOffset),this.target.sub(this.cursor),this.target.clampLength(this.minTargetRadius,this.maxTargetRadius),this.target.add(this.cursor);let r=!1;if(this.zoomToCursor&&this._performCursorZoom||this.object.isOrthographicCamera)this._spherical.radius=this._clampDistance(this._spherical.radius);else{let o=this._spherical.radius;this._spherical.radius=this._clampDistance(this._spherical.radius*this._scale),r=o!=this._spherical.radius}if(Ue.setFromSpherical(this._spherical),Ue.applyQuaternion(this._quatInverse),e.copy(this.target).add(Ue),this.object.lookAt(this.target),this.enableDamping===!0?(this._sphericalDelta.theta*=1-this.dampingFactor,this._sphericalDelta.phi*=1-this.dampingFactor,this._panOffset.multiplyScalar(1-this.dampingFactor)):(this._sphericalDelta.set(0,0,0),this._panOffset.set(0,0,0)),this.zoomToCursor&&this._performCursorZoom){let o=null;if(this.object.isPerspectiveCamera){let a=Ue.length();o=this._clampDistance(a*this._scale);let c=a-o;this.object.position.addScaledVector(this._dollyDirection,c),this.object.updateMatrixWorld(),r=!!c}else if(this.object.isOrthographicCamera){let a=new U(this._mouse.x,this._mouse.y,0);a.unproject(this.object);let c=this.object.zoom;this.object.zoom=Math.max(this.minZoom,Math.min(this.maxZoom,this.object.zoom/this._scale)),this.object.updateProjectionMatrix(),r=c!==this.object.zoom;let l=new U(this._mouse.x,this._mouse.y,0);l.unproject(this.object),this.object.position.sub(l).add(a),this.object.updateMatrixWorld(),o=Ue.length()}else console.warn("WARNING: OrbitControls.js encountered an unknown camera type - zoom to cursor disabled."),this.zoomToCursor=!1;o!==null&&(this.screenSpacePanning?this.target.set(0,0,-1).transformDirection(this.object.matrix).multiplyScalar(o).add(this.object.position):(oo.origin.copy(this.object.position),oo.direction.set(0,0,-1).transformDirection(this.object.matrix),Math.abs(this.object.up.dot(oo.direction))<T0?this.object.lookAt(this.target):(vh.setFromNormalAndCoplanarPoint(this.object.up,this.target),oo.intersectPlane(vh,this.target))))}else if(this.object.isOrthographicCamera){let o=this.object.zoom;this.object.zoom=Math.max(this.minZoom,Math.min(this.maxZoom,this.object.zoom/this._scale)),o!==this.object.zoom&&(this.object.updateProjectionMatrix(),r=!0)}return this._scale=1,this._performCursorZoom=!1,r||this._lastPosition.distanceToSquared(this.object.position)>wc||8*(1-this._lastQuaternion.dot(this.object.quaternion))>wc||this._lastTargetPosition.distanceToSquared(this.target)>wc?(this.dispatchEvent(yh),this._lastPosition.copy(this.object.position),this._lastQuaternion.copy(this.object.quaternion),this._lastTargetPosition.copy(this.target),!0):!1}_getAutoRotationAngle(t){return t!==null?rn/60*this.autoRotateSpeed*t:rn/60/60*this.autoRotateSpeed}_getZoomScale(t){let e=Math.abs(t*.01);return Math.pow(.95,this.zoomSpeed*e)}_rotateLeft(t){this._sphericalDelta.theta-=t}_rotateUp(t){this._sphericalDelta.phi-=t}_panLeft(t,e){Ue.setFromMatrixColumn(e,0),Ue.multiplyScalar(-t),this._panOffset.add(Ue)}_panUp(t,e){this.screenSpacePanning===!0?Ue.setFromMatrixColumn(e,1):(Ue.setFromMatrixColumn(e,0),Ue.crossVectors(this.object.up,Ue)),Ue.multiplyScalar(t),this._panOffset.add(Ue)}_pan(t,e){let n=this.domElement;if(this.object.isPerspectiveCamera){let s=this.object.position;Ue.copy(s).sub(this.target);let r=Ue.length();r*=Math.tan(this.object.fov/2*Math.PI/180),this._panLeft(2*t*r/n.clientHeight,this.object.matrix),this._panUp(2*e*r/n.clientHeight,this.object.matrix)}else this.object.isOrthographicCamera?(this._panLeft(t*(this.object.right-this.object.left)/this.object.zoom/n.clientWidth,this.object.matrix),this._panUp(e*(this.object.top-this.object.bottom)/this.object.zoom/n.clientHeight,this.object.matrix)):(console.warn("WARNING: OrbitControls.js encountered an unknown camera type - pan disabled."),this.enablePan=!1)}_dollyOut(t){this.object.isPerspectiveCamera||this.object.isOrthographicCamera?this._scale/=t:(console.warn("WARNING: OrbitControls.js encountered an unknown camera type - dolly/zoom disabled."),this.enableZoom=!1)}_dollyIn(t){this.object.isPerspectiveCamera||this.object.isOrthographicCamera?this._scale*=t:(console.warn("WARNING: OrbitControls.js encountered an unknown camera type - dolly/zoom disabled."),this.enableZoom=!1)}_updateZoomParameters(t,e){if(!this.zoomToCursor)return;this._performCursorZoom=!0;let n=this.domElement.getBoundingClientRect(),s=t-n.left,r=e-n.top,o=n.width,a=n.height;this._mouse.x=s/o*2-1,this._mouse.y=-(r/a)*2+1,this._dollyDirection.set(this._mouse.x,this._mouse.y,1).unproject(this.object).sub(this.object.position).normalize()}_clampDistance(t){return Math.max(this.minDistance,Math.min(this.maxDistance,t))}_handleMouseDownRotate(t){this._rotateStart.set(t.clientX,t.clientY)}_handleMouseDownDolly(t){this._updateZoomParameters(t.clientX,t.clientX),this._dollyStart.set(t.clientX,t.clientY)}_handleMouseDownPan(t){this._panStart.set(t.clientX,t.clientY)}_handleMouseMoveRotate(t){this._rotateEnd.set(t.clientX,t.clientY),this._rotateDelta.subVectors(this._rotateEnd,this._rotateStart).multiplyScalar(this.rotateSpeed);let e=this.domElement;this._rotateLeft(rn*this._rotateDelta.x/e.clientHeight),this._rotateUp(rn*this._rotateDelta.y/e.clientHeight),this._rotateStart.copy(this._rotateEnd),this.update()}_handleMouseMoveDolly(t){this._dollyEnd.set(t.clientX,t.clientY),this._dollyDelta.subVectors(this._dollyEnd,this._dollyStart),this._dollyDelta.y>0?this._dollyOut(this._getZoomScale(this._dollyDelta.y)):this._dollyDelta.y<0&&this._dollyIn(this._getZoomScale(this._dollyDelta.y)),this._dollyStart.copy(this._dollyEnd),this.update()}_handleMouseMovePan(t){this._panEnd.set(t.clientX,t.clientY),this._panDelta.subVectors(this._panEnd,this._panStart).multiplyScalar(this.panSpeed),this._pan(this._panDelta.x,this._panDelta.y),this._panStart.copy(this._panEnd),this.update()}_handleMouseWheel(t){this._updateZoomParameters(t.clientX,t.clientY),t.deltaY<0?this._dollyIn(this._getZoomScale(t.deltaY)):t.deltaY>0&&this._dollyOut(this._getZoomScale(t.deltaY)),this.update()}_handleKeyDown(t){let e=!1;switch(t.code){case this.keys.UP:t.ctrlKey||t.metaKey||t.shiftKey?this._rotateUp(rn*this.rotateSpeed/this.domElement.clientHeight):this._pan(0,this.keyPanSpeed),e=!0;break;case this.keys.BOTTOM:t.ctrlKey||t.metaKey||t.shiftKey?this._rotateUp(-rn*this.rotateSpeed/this.domElement.clientHeight):this._pan(0,-this.keyPanSpeed),e=!0;break;case this.keys.LEFT:t.ctrlKey||t.metaKey||t.shiftKey?this._rotateLeft(rn*this.rotateSpeed/this.domElement.clientHeight):this._pan(this.keyPanSpeed,0),e=!0;break;case this.keys.RIGHT:t.ctrlKey||t.metaKey||t.shiftKey?this._rotateLeft(-rn*this.rotateSpeed/this.domElement.clientHeight):this._pan(-this.keyPanSpeed,0),e=!0;break}e&&(t.preventDefault(),this.update())}_handleTouchStartRotate(t){if(this._pointers.length===1)this._rotateStart.set(t.pageX,t.pageY);else{let e=this._getSecondPointerPosition(t),n=.5*(t.pageX+e.x),s=.5*(t.pageY+e.y);this._rotateStart.set(n,s)}}_handleTouchStartPan(t){if(this._pointers.length===1)this._panStart.set(t.pageX,t.pageY);else{let e=this._getSecondPointerPosition(t),n=.5*(t.pageX+e.x),s=.5*(t.pageY+e.y);this._panStart.set(n,s)}}_handleTouchStartDolly(t){let e=this._getSecondPointerPosition(t),n=t.pageX-e.x,s=t.pageY-e.y,r=Math.sqrt(n*n+s*s);this._dollyStart.set(0,r)}_handleTouchStartDollyPan(t){this.enableZoom&&this._handleTouchStartDolly(t),this.enablePan&&this._handleTouchStartPan(t)}_handleTouchStartDollyRotate(t){this.enableZoom&&this._handleTouchStartDolly(t),this.enableRotate&&this._handleTouchStartRotate(t)}_handleTouchMoveRotate(t){if(this._pointers.length==1)this._rotateEnd.set(t.pageX,t.pageY);else{let n=this._getSecondPointerPosition(t),s=.5*(t.pageX+n.x),r=.5*(t.pageY+n.y);this._rotateEnd.set(s,r)}this._rotateDelta.subVectors(this._rotateEnd,this._rotateStart).multiplyScalar(this.rotateSpeed);let e=this.domElement;this._rotateLeft(rn*this._rotateDelta.x/e.clientHeight),this._rotateUp(rn*this._rotateDelta.y/e.clientHeight),this._rotateStart.copy(this._rotateEnd)}_handleTouchMovePan(t){if(this._pointers.length===1)this._panEnd.set(t.pageX,t.pageY);else{let e=this._getSecondPointerPosition(t),n=.5*(t.pageX+e.x),s=.5*(t.pageY+e.y);this._panEnd.set(n,s)}this._panDelta.subVectors(this._panEnd,this._panStart).multiplyScalar(this.panSpeed),this._pan(this._panDelta.x,this._panDelta.y),this._panStart.copy(this._panEnd)}_handleTouchMoveDolly(t){let e=this._getSecondPointerPosition(t),n=t.pageX-e.x,s=t.pageY-e.y,r=Math.sqrt(n*n+s*s);this._dollyEnd.set(0,r),this._dollyDelta.set(0,Math.pow(this._dollyEnd.y/this._dollyStart.y,this.zoomSpeed)),this._dollyOut(this._dollyDelta.y),this._dollyStart.copy(this._dollyEnd);let o=(t.pageX+e.x)*.5,a=(t.pageY+e.y)*.5;this._updateZoomParameters(o,a)}_handleTouchMoveDollyPan(t){this.enableZoom&&this._handleTouchMoveDolly(t),this.enablePan&&this._handleTouchMovePan(t)}_handleTouchMoveDollyRotate(t){this.enableZoom&&this._handleTouchMoveDolly(t),this.enableRotate&&this._handleTouchMoveRotate(t)}_addPointer(t){this._pointers.push(t.pointerId)}_removePointer(t){delete this._pointerPositions[t.pointerId];for(let e=0;e<this._pointers.length;e++)if(this._pointers[e]==t.pointerId){this._pointers.splice(e,1);return}}_isTrackingPointer(t){for(let e=0;e<this._pointers.length;e++)if(this._pointers[e]==t.pointerId)return!0;return!1}_trackPointer(t){let e=this._pointerPositions[t.pointerId];e===void 0&&(e=new Wt,this._pointerPositions[t.pointerId]=e),e.set(t.pageX,t.pageY)}_getSecondPointerPosition(t){let e=t.pointerId===this._pointers[0]?this._pointers[1]:this._pointers[0];return this._pointerPositions[e]}_customWheelEvent(t){let e=t.deltaMode,n={clientX:t.clientX,clientY:t.clientY,deltaY:t.deltaY};switch(e){case 1:n.deltaY*=16;break;case 2:n.deltaY*=100;break}return t.ctrlKey&&!this._controlActive&&(n.deltaY*=10),n}};function w0(i){this.enabled!==!1&&(this._pointers.length===0&&(this.domElement.setPointerCapture(i.pointerId),this.domElement.addEventListener("pointermove",this._onPointerMove),this.domElement.addEventListener("pointerup",this._onPointerUp)),!this._isTrackingPointer(i)&&(this._addPointer(i),i.pointerType==="touch"?this._onTouchStart(i):this._onMouseDown(i)))}function A0(i){this.enabled!==!1&&(i.pointerType==="touch"?this._onTouchMove(i):this._onMouseMove(i))}function C0(i){switch(this._removePointer(i),this._pointers.length){case 0:this.domElement.releasePointerCapture(i.pointerId),this.domElement.removeEventListener("pointermove",this._onPointerMove),this.domElement.removeEventListener("pointerup",this._onPointerUp),this.dispatchEvent(Mh),this.state=ye.NONE;break;case 1:let t=this._pointers[0],e=this._pointerPositions[t];this._onTouchStart({pointerId:t,pageX:e.x,pageY:e.y});break}}function R0(i){let t;switch(i.button){case 0:t=this.mouseButtons.LEFT;break;case 1:t=this.mouseButtons.MIDDLE;break;case 2:t=this.mouseButtons.RIGHT;break;default:t=-1}switch(t){case Li.DOLLY:if(this.enableZoom===!1)return;this._handleMouseDownDolly(i),this.state=ye.DOLLY;break;case Li.ROTATE:if(i.ctrlKey||i.metaKey||i.shiftKey){if(this.enablePan===!1)return;this._handleMouseDownPan(i),this.state=ye.PAN}else{if(this.enableRotate===!1)return;this._handleMouseDownRotate(i),this.state=ye.ROTATE}break;case Li.PAN:if(i.ctrlKey||i.metaKey||i.shiftKey){if(this.enableRotate===!1)return;this._handleMouseDownRotate(i),this.state=ye.ROTATE}else{if(this.enablePan===!1)return;this._handleMouseDownPan(i),this.state=ye.PAN}break;default:this.state=ye.NONE}this.state!==ye.NONE&&this.dispatchEvent(Ac)}function P0(i){switch(this.state){case ye.ROTATE:if(this.enableRotate===!1)return;this._handleMouseMoveRotate(i);break;case ye.DOLLY:if(this.enableZoom===!1)return;this._handleMouseMoveDolly(i);break;case ye.PAN:if(this.enablePan===!1)return;this._handleMouseMovePan(i);break}}function I0(i){this.enabled===!1||this.enableZoom===!1||this.state!==ye.NONE||(i.preventDefault(),this.dispatchEvent(Ac),this._handleMouseWheel(this._customWheelEvent(i)),this.dispatchEvent(Mh))}function L0(i){this.enabled===!1||this.enablePan===!1||this._handleKeyDown(i)}function D0(i){switch(this._trackPointer(i),this._pointers.length){case 1:switch(this.touches.ONE){case Di.ROTATE:if(this.enableRotate===!1)return;this._handleTouchStartRotate(i),this.state=ye.TOUCH_ROTATE;break;case Di.PAN:if(this.enablePan===!1)return;this._handleTouchStartPan(i),this.state=ye.TOUCH_PAN;break;default:this.state=ye.NONE}break;case 2:switch(this.touches.TWO){case Di.DOLLY_PAN:if(this.enableZoom===!1&&this.enablePan===!1)return;this._handleTouchStartDollyPan(i),this.state=ye.TOUCH_DOLLY_PAN;break;case Di.DOLLY_ROTATE:if(this.enableZoom===!1&&this.enableRotate===!1)return;this._handleTouchStartDollyRotate(i),this.state=ye.TOUCH_DOLLY_ROTATE;break;default:this.state=ye.NONE}break;default:this.state=ye.NONE}this.state!==ye.NONE&&this.dispatchEvent(Ac)}function U0(i){switch(this._trackPointer(i),this.state){case ye.TOUCH_ROTATE:if(this.enableRotate===!1)return;this._handleTouchMoveRotate(i),this.update();break;case ye.TOUCH_PAN:if(this.enablePan===!1)return;this._handleTouchMovePan(i),this.update();break;case ye.TOUCH_DOLLY_PAN:if(this.enableZoom===!1&&this.enablePan===!1)return;this._handleTouchMoveDollyPan(i),this.update();break;case ye.TOUCH_DOLLY_ROTATE:if(this.enableZoom===!1&&this.enableRotate===!1)return;this._handleTouchMoveDollyRotate(i),this.update();break;default:this.state=ye.NONE}}function N0(i){this.enabled!==!1&&i.preventDefault()}function F0(i){i.key==="Control"&&(this._controlActive=!0,this.domElement.getRootNode().addEventListener("keyup",this._interceptControlUp,{passive:!0,capture:!0}))}function O0(i){i.key==="Control"&&(this._controlActive=!1,this.domElement.getRootNode().removeEventListener("keyup",this._interceptControlUp,{passive:!0,capture:!0}))}var Ms=class i extends se{constructor(){let t=i.SkyShader,e=new Pe({name:t.name,uniforms:xs.clone(t.uniforms),vertexShader:t.vertexShader,fragmentShader:t.fragmentShader,side:ke,depthWrite:!1});super(new Qe(1,1,1),e),this.isSky=!0}};Ms.SkyShader={name:"SkyShader",uniforms:{turbidity:{value:2},rayleigh:{value:1},mieCoefficient:{value:.005},mieDirectionalG:{value:.8},sunPosition:{value:new U},up:{value:new U(0,1,0)}},vertexShader:`
		uniform vec3 sunPosition;
		uniform float rayleigh;
		uniform float turbidity;
		uniform float mieCoefficient;
		uniform vec3 up;

		varying vec3 vWorldPosition;
		varying vec3 vSunDirection;
		varying float vSunfade;
		varying vec3 vBetaR;
		varying vec3 vBetaM;
		varying float vSunE;

		// constants for atmospheric scattering
		const float e = 2.71828182845904523536028747135266249775724709369995957;
		const float pi = 3.141592653589793238462643383279502884197169;

		// wavelength of used primaries, according to preetham
		const vec3 lambda = vec3( 680E-9, 550E-9, 450E-9 );
		// this pre-calcuation replaces older TotalRayleigh(vec3 lambda) function:
		// (8.0 * pow(pi, 3.0) * pow(pow(n, 2.0) - 1.0, 2.0) * (6.0 + 3.0 * pn)) / (3.0 * N * pow(lambda, vec3(4.0)) * (6.0 - 7.0 * pn))
		const vec3 totalRayleigh = vec3( 5.804542996261093E-6, 1.3562911419845635E-5, 3.0265902468824876E-5 );

		// mie stuff
		// K coefficient for the primaries
		const float v = 4.0;
		const vec3 K = vec3( 0.686, 0.678, 0.666 );
		// MieConst = pi * pow( ( 2.0 * pi ) / lambda, vec3( v - 2.0 ) ) * K
		const vec3 MieConst = vec3( 1.8399918514433978E14, 2.7798023919660528E14, 4.0790479543861094E14 );

		// earth shadow hack
		// cutoffAngle = pi / 1.95;
		const float cutoffAngle = 1.6110731556870734;
		const float steepness = 1.5;
		const float EE = 1000.0;

		float sunIntensity( float zenithAngleCos ) {
			zenithAngleCos = clamp( zenithAngleCos, -1.0, 1.0 );
			return EE * max( 0.0, 1.0 - pow( e, -( ( cutoffAngle - acos( zenithAngleCos ) ) / steepness ) ) );
		}

		vec3 totalMie( float T ) {
			float c = ( 0.2 * T ) * 10E-18;
			return 0.434 * c * MieConst;
		}

		void main() {

			vec4 worldPosition = modelMatrix * vec4( position, 1.0 );
			vWorldPosition = worldPosition.xyz;

			gl_Position = projectionMatrix * modelViewMatrix * vec4( position, 1.0 );
			gl_Position.z = gl_Position.w; // set z to camera.far

			vSunDirection = normalize( sunPosition );

			vSunE = sunIntensity( dot( vSunDirection, up ) );

			vSunfade = 1.0 - clamp( 1.0 - exp( ( sunPosition.y / 450000.0 ) ), 0.0, 1.0 );

			float rayleighCoefficient = rayleigh - ( 1.0 * ( 1.0 - vSunfade ) );

			// extinction (absorbtion + out scattering)
			// rayleigh coefficients
			vBetaR = totalRayleigh * rayleighCoefficient;

			// mie coefficients
			vBetaM = totalMie( turbidity ) * mieCoefficient;

		}`,fragmentShader:`
		varying vec3 vWorldPosition;
		varying vec3 vSunDirection;
		varying float vSunfade;
		varying vec3 vBetaR;
		varying vec3 vBetaM;
		varying float vSunE;

		uniform float mieDirectionalG;
		uniform vec3 up;

		// constants for atmospheric scattering
		const float pi = 3.141592653589793238462643383279502884197169;

		const float n = 1.0003; // refractive index of air
		const float N = 2.545E25; // number of molecules per unit volume for air at 288.15K and 1013mb (sea level -45 celsius)

		// optical length at zenith for molecules
		const float rayleighZenithLength = 8.4E3;
		const float mieZenithLength = 1.25E3;
		// 66 arc seconds -> degrees, and the cosine of that
		const float sunAngularDiameterCos = 0.999956676946448443553574619906976478926848692873900859324;

		// 3.0 / ( 16.0 * pi )
		const float THREE_OVER_SIXTEENPI = 0.05968310365946075;
		// 1.0 / ( 4.0 * pi )
		const float ONE_OVER_FOURPI = 0.07957747154594767;

		float rayleighPhase( float cosTheta ) {
			return THREE_OVER_SIXTEENPI * ( 1.0 + pow( cosTheta, 2.0 ) );
		}

		float hgPhase( float cosTheta, float g ) {
			float g2 = pow( g, 2.0 );
			float inverse = 1.0 / pow( 1.0 - 2.0 * g * cosTheta + g2, 1.5 );
			return ONE_OVER_FOURPI * ( ( 1.0 - g2 ) * inverse );
		}

		void main() {

			vec3 direction = normalize( vWorldPosition - cameraPosition );

			// optical length
			// cutoff angle at 90 to avoid singularity in next formula.
			float zenithAngle = acos( max( 0.0, dot( up, direction ) ) );
			float inverse = 1.0 / ( cos( zenithAngle ) + 0.15 * pow( 93.885 - ( ( zenithAngle * 180.0 ) / pi ), -1.253 ) );
			float sR = rayleighZenithLength * inverse;
			float sM = mieZenithLength * inverse;

			// combined extinction factor
			vec3 Fex = exp( -( vBetaR * sR + vBetaM * sM ) );

			// in scattering
			float cosTheta = dot( direction, vSunDirection );

			float rPhase = rayleighPhase( cosTheta * 0.5 + 0.5 );
			vec3 betaRTheta = vBetaR * rPhase;

			float mPhase = hgPhase( cosTheta, mieDirectionalG );
			vec3 betaMTheta = vBetaM * mPhase;

			vec3 Lin = pow( vSunE * ( ( betaRTheta + betaMTheta ) / ( vBetaR + vBetaM ) ) * ( 1.0 - Fex ), vec3( 1.5 ) );
			Lin *= mix( vec3( 1.0 ), pow( vSunE * ( ( betaRTheta + betaMTheta ) / ( vBetaR + vBetaM ) ) * Fex, vec3( 1.0 / 2.0 ) ), clamp( pow( 1.0 - dot( up, vSunDirection ), 5.0 ), 0.0, 1.0 ) );

			// nightsky
			float theta = acos( direction.y ); // elevation --> y-axis, [-pi/2, pi/2]
			float phi = atan( direction.z, direction.x ); // azimuth --> x-axis [-pi/2, pi/2]
			vec2 uv = vec2( phi, theta ) / vec2( 2.0 * pi, pi ) + vec2( 0.5, 0.0 );
			vec3 L0 = vec3( 0.1 ) * Fex;

			// composition + solar disc
			float sundisk = smoothstep( sunAngularDiameterCos, sunAngularDiameterCos + 0.00002, cosTheta );
			L0 += ( vSunE * 19000.0 * Fex ) * sundisk;

			vec3 texColor = ( Lin + L0 ) * 0.04 + vec3( 0.0, 0.0003, 0.00075 );

			vec3 retColor = pow( texColor, vec3( 1.0 / ( 1.2 + ( 1.2 * vSunfade ) ) ) );

			gl_FragColor = vec4( retColor, 1.0 );

			#include <tonemapping_fragment>
			#include <colorspace_fragment>

		}`};var co=class extends se{constructor(t,e={}){super(t),this.isWater=!0;let n=this,s=e.textureWidth!==void 0?e.textureWidth:512,r=e.textureHeight!==void 0?e.textureHeight:512,o=e.clipBias!==void 0?e.clipBias:0,a=e.alpha!==void 0?e.alpha:1,c=e.time!==void 0?e.time:0,l=e.waterNormals!==void 0?e.waterNormals:null,h=e.sunDirection!==void 0?e.sunDirection:new U(.70707,.70707,0),u=new _t(e.sunColor!==void 0?e.sunColor:16777215),p=new _t(e.waterColor!==void 0?e.waterColor:8355711),f=e.eye!==void 0?e.eye:new U(0,0,0),g=e.distortionScale!==void 0?e.distortionScale:20,y=e.side!==void 0?e.side:Dn,m=e.fog!==void 0?e.fog:!1,d=new un,x=new U,_=new U,v=new U,C=new ce,w=new U(0,0,-1),A=new be,P=new U,E=new U,M=new be,R=new ce,F=new Ve,V=new En(s,r),z={name:"MirrorShader",uniforms:xs.merge([Mt.fog,Mt.lights,{normalSampler:{value:null},mirrorSampler:{value:null},alpha:{value:1},time:{value:0},size:{value:1},distortionScale:{value:20},textureMatrix:{value:new ce},sunColor:{value:new _t(8355711)},sunDirection:{value:new U(.70707,.70707,0)},eye:{value:new U},waterColor:{value:new _t(5592405)}}]),vertexShader:`
				uniform mat4 textureMatrix;
				uniform float time;

				varying vec4 mirrorCoord;
				varying vec4 worldPosition;

				#include <common>
				#include <fog_pars_vertex>
				#include <shadowmap_pars_vertex>
				#include <logdepthbuf_pars_vertex>

				void main() {
					mirrorCoord = modelMatrix * vec4( position, 1.0 );
					worldPosition = mirrorCoord.xyzw;
					mirrorCoord = textureMatrix * mirrorCoord;
					vec4 mvPosition =  modelViewMatrix * vec4( position, 1.0 );
					gl_Position = projectionMatrix * mvPosition;

				#include <beginnormal_vertex>
				#include <defaultnormal_vertex>
				#include <logdepthbuf_vertex>
				#include <fog_vertex>
				#include <shadowmap_vertex>
			}`,fragmentShader:`
				uniform sampler2D mirrorSampler;
				uniform float alpha;
				uniform float time;
				uniform float size;
				uniform float distortionScale;
				uniform sampler2D normalSampler;
				uniform vec3 sunColor;
				uniform vec3 sunDirection;
				uniform vec3 eye;
				uniform vec3 waterColor;

				varying vec4 mirrorCoord;
				varying vec4 worldPosition;

				vec4 getNoise( vec2 uv ) {
					vec2 uv0 = ( uv / 103.0 ) + vec2(time / 17.0, time / 29.0);
					vec2 uv1 = uv / 107.0-vec2( time / -19.0, time / 31.0 );
					vec2 uv2 = uv / vec2( 8907.0, 9803.0 ) + vec2( time / 101.0, time / 97.0 );
					vec2 uv3 = uv / vec2( 1091.0, 1027.0 ) - vec2( time / 109.0, time / -113.0 );
					vec4 noise = texture2D( normalSampler, uv0 ) +
						texture2D( normalSampler, uv1 ) +
						texture2D( normalSampler, uv2 ) +
						texture2D( normalSampler, uv3 );
					return noise * 0.5 - 1.0;
				}

				void sunLight( const vec3 surfaceNormal, const vec3 eyeDirection, float shiny, float spec, float diffuse, inout vec3 diffuseColor, inout vec3 specularColor ) {
					vec3 reflection = normalize( reflect( -sunDirection, surfaceNormal ) );
					float direction = max( 0.0, dot( eyeDirection, reflection ) );
					specularColor += pow( direction, shiny ) * sunColor * spec;
					diffuseColor += max( dot( sunDirection, surfaceNormal ), 0.0 ) * sunColor * diffuse;
				}

				#include <common>
				#include <packing>
				#include <bsdfs>
				#include <fog_pars_fragment>
				#include <logdepthbuf_pars_fragment>
				#include <lights_pars_begin>
				#include <shadowmap_pars_fragment>
				#include <shadowmask_pars_fragment>

				void main() {

					#include <logdepthbuf_fragment>
					vec4 noise = getNoise( worldPosition.xz * size );
					vec3 surfaceNormal = normalize( noise.xzy * vec3( 1.5, 1.0, 1.5 ) );

					vec3 diffuseLight = vec3(0.0);
					vec3 specularLight = vec3(0.0);

					vec3 worldToEye = eye-worldPosition.xyz;
					vec3 eyeDirection = normalize( worldToEye );
					sunLight( surfaceNormal, eyeDirection, 100.0, 2.0, 0.5, diffuseLight, specularLight );

					float distance = length(worldToEye);

					vec2 distortion = surfaceNormal.xz * ( 0.001 + 1.0 / distance ) * distortionScale;
					vec3 reflectionSample = vec3( texture2D( mirrorSampler, mirrorCoord.xy / mirrorCoord.w + distortion ) );

					float theta = max( dot( eyeDirection, surfaceNormal ), 0.0 );
					float rf0 = 0.3;
					float reflectance = rf0 + ( 1.0 - rf0 ) * pow( ( 1.0 - theta ), 5.0 );
					vec3 scatter = max( 0.0, dot( surfaceNormal, eyeDirection ) ) * waterColor;
					vec3 albedo = mix( ( sunColor * diffuseLight * 0.3 + scatter ) * getShadowMask(), ( vec3( 0.1 ) + reflectionSample * 0.9 + reflectionSample * specularLight ), reflectance);
					vec3 outgoingLight = albedo;
					gl_FragColor = vec4( outgoingLight, alpha );

					#include <tonemapping_fragment>
					#include <colorspace_fragment>
					#include <fog_fragment>	
				}`},k=new Pe({name:z.name,uniforms:xs.clone(z.uniforms),vertexShader:z.vertexShader,fragmentShader:z.fragmentShader,lights:!0,side:y,fog:m});k.uniforms.mirrorSampler.value=V.texture,k.uniforms.textureMatrix.value=R,k.uniforms.alpha.value=a,k.uniforms.time.value=c,k.uniforms.normalSampler.value=l,k.uniforms.sunColor.value=u,k.uniforms.waterColor.value=p,k.uniforms.sunDirection.value=h,k.uniforms.distortionScale.value=g,k.uniforms.eye.value=f,n.material=k,n.onBeforeRender=function(I,Y,B){if(_.setFromMatrixPosition(n.matrixWorld),v.setFromMatrixPosition(B.matrixWorld),C.extractRotation(n.matrixWorld),x.set(0,0,1),x.applyMatrix4(C),P.subVectors(_,v),P.dot(x)>0)return;P.reflect(x).negate(),P.add(_),C.extractRotation(B.matrixWorld),w.set(0,0,-1),w.applyMatrix4(C),w.add(v),E.subVectors(_,w),E.reflect(x).negate(),E.add(_),F.position.copy(P),F.up.set(0,1,0),F.up.applyMatrix4(C),F.up.reflect(x),F.lookAt(E),F.far=B.far,F.updateMatrixWorld(),F.projectionMatrix.copy(B.projectionMatrix),R.set(.5,0,0,.5,0,.5,0,.5,0,0,.5,.5,0,0,0,1),R.multiply(F.projectionMatrix),R.multiply(F.matrixWorldInverse),d.setFromNormalAndCoplanarPoint(x,_),d.applyMatrix4(F.matrixWorldInverse),A.set(d.normal.x,d.normal.y,d.normal.z,d.constant);let Q=F.projectionMatrix;M.x=(Math.sign(A.x)+Q.elements[8])/Q.elements[0],M.y=(Math.sign(A.y)+Q.elements[9])/Q.elements[5],M.z=-1,M.w=(1+Q.elements[10])/Q.elements[14],A.multiplyScalar(2/A.dot(M)),Q.elements[2]=A.x,Q.elements[6]=A.y,Q.elements[10]=A.z+1-o,Q.elements[14]=A.w,f.setFromMatrixPosition(B.matrixWorld);let ut=I.getRenderTarget(),Et=I.xr.enabled,zt=I.shadowMap.autoUpdate;n.visible=!1,I.xr.enabled=!1,I.shadowMap.autoUpdate=!1,I.setRenderTarget(V),I.state.buffers.depth.setMask(!0),I.autoClear===!1&&I.clear(),I.render(Y,F),n.visible=!0,I.xr.enabled=Et,I.shadowMap.autoUpdate=zt,I.setRenderTarget(ut);let $t=B.viewport;$t!==void 0&&I.state.viewport($t)}}};function tn(i,t,e){let n=Math.min(1,Math.max(0,(e-i)/(t-i)));return n*n*(3-2*n)}function _i(i,t,e){return i<t?t:i>e?e:i}function Cc(i,t,e){return i+(t-i)*e}function Ne(i){let t=i>>>0;return function(){t|=0,t=t+1831565813|0;let e=Math.imul(t^t>>>15,1|t);return e=e+Math.imul(e^e>>>7,61|e)^e,((e^e>>>14)>>>0)/4294967296}}var ti=new Uint8Array(512);(function(){let t=Ne(1337),e=new Uint8Array(256);for(let n=0;n<256;n++)e[n]=n;for(let n=255;n>0;n--){let s=Math.floor(t()*(n+1)),r=e[n];e[n]=e[s],e[s]=r}for(let n=0;n<512;n++)ti[n]=e[n&255]})();function Sh(i){return i*i*i*(i*(i*6-15)+10)}function lo(i,t,e){switch(i&7){case 0:return t+e;case 1:return-t+e;case 2:return t-e;case 3:return-t-e;case 4:return t;case 5:return-t;case 6:return e;default:return-e}}function Eh(i,t){let e=Math.floor(i)&255,n=Math.floor(t)&255,s=i-Math.floor(i),r=t-Math.floor(t),o=Sh(s),a=Sh(r),c=ti[ti[e]+n],l=ti[ti[e]+n+1],h=ti[ti[e+1]+n],u=ti[ti[e+1]+n+1],p=Cc(lo(c,s,r),lo(h,s-1,r),o),f=Cc(lo(l,s,r-1),lo(u,s-1,r-1),o);return Cc(p,f,a)*.70710678}function bh(i,t,e=5,n=2,s=.5){let r=.5,o=1,a=0;for(let c=0;c<e;c++)a+=Eh(i*o,t*o)*r,o*=n,r*=s;return a}function B0(i,t,e=5){let n=.55,s=1,r=0,o=0;for(let a=0;a<e;a++){let c=1-Math.abs(Eh(i*s,t*s)*2);r+=c*c*n,o+=n,s*=2.03,n*=.5}return r/o}function _e(i,t){let e=Math.hypot(i,t),n=Math.atan2(t,i),r=-9.5+12.5*tn(105,240,e),o=bh(i*.0055+13.1,t*.0055-4.7,4)*2;r+=o*5.5*tn(120,255,e);let a=tn(330,730,e),c=B0(i*.0028+2.2,t*.0028-1.7,5),l=.48+.52*(.5+.5*Math.cos(n-2.15));r+=Math.pow(a,1.35)*(26+165*c)*l;let h=tn(55,135,r)*(bh(i*.021,t*.021,3)*2)*13;return r+=h,r+=tn(650,900,e)*24,r}function xi(i,t){let n=_e(i+3,t)-_e(i-3,t),s=_e(i,t+3)-_e(i,t-3);return Math.hypot(n,s)/(2*3)}function Zs(i,t){let e=_e(i,t);if(e<0+1.2||e>78)return 0;let n=xi(i,t),s=tn(230,120,Math.hypot(i,t))*.35+.65;return _i(tn(0+1.2,7,e)*(1-tn(.42,.85,n))*(1-tn(58,88,e))*s,0,1)}var z0=`
varying vec3 vDir;
void main() {
  vDir = normalize(position);
  gl_Position = projectionMatrix * modelViewMatrix * vec4(position, 1.0);
}`,k0=`
uniform float uTime;
uniform float uCoverage;
uniform float uOpacity;
uniform float uSun;
uniform vec3 uLit;
uniform vec3 uShadow;
uniform vec3 uSunDir;
varying vec3 vDir;

float hash(vec2 p) {
  return fract(sin(dot(p, vec2(127.1, 311.7))) * 43758.5453);
}
float vnoise(vec2 p) {
  vec2 i = floor(p);
  vec2 f = fract(p);
  f = f * f * (3.0 - 2.0 * f);
  float a = hash(i);
  float b = hash(i + vec2(1.0, 0.0));
  float c = hash(i + vec2(0.0, 1.0));
  float d = hash(i + vec2(1.0, 1.0));
  return mix(mix(a, b, f.x), mix(c, d, f.x), f.y);
}
float fbm(vec2 p) {
  float v = 0.0;
  float a = 0.55;
  for (int i = 0; i < 4; i++) {
    v += vnoise(p) * a;
    p = p * 2.07 + vec2(17.1, 9.2);
    a *= 0.5;
  }
  return v;
}
void main() {
  vec3 d = normalize(vDir);
  float up = smoothstep(0.015, 0.20, d.y);
  vec2 p = d.xz / max(d.y, 0.015) * 0.72;
  vec2 wind = vec2(uTime * 0.0035, uTime * 0.0012);
  float n = fbm(p + wind);
  float detail = fbm(p * 3.1 - wind * 1.7 + vec2(11.0, 4.0));
  float shape = n * 0.76 + detail * 0.24;
  float cover = smoothstep(uCoverage, uCoverage + 0.20, shape);
  float alpha = cover * up * uOpacity;
  if (alpha < 0.004) discard;
  float lit = smoothstep(0.32, 0.82, shape + detail * 0.16);
  vec3 col = mix(uShadow, uLit, lit * (0.55 + 0.45 * uSun));
  float sun = pow(max(dot(d, normalize(uSunDir)), 0.0), 9.0);
  col += uLit * sun * 0.22 * up;
  gl_FragColor = vec4(col, alpha);
}`;function H0(){let i=new Pe({uniforms:{uTime:{value:0},uCoverage:{value:.55},uOpacity:{value:.62},uSun:{value:1},uLit:{value:new _t(16777215)},uShadow:{value:new _t(8885930)},uSunDir:{value:new U(.3,.8,.2)}},vertexShader:z0,fragmentShader:k0,transparent:!0,depthWrite:!1,depthTest:!0,side:ke,fog:!1}),t=new se(new $e(7200,32,20),i);return t.renderOrder=-940,t.frustumCulled=!1,t}function V0(i,t,e,n){let s=Math.floor(i),r=Math.floor(t),o=i-s,a=t-r,c=o*o*(3-2*o),l=a*a*(3-2*a),h=(y,m)=>{let d=Math.imul((y%e+e)%e+31,374761393)^Math.imul((m%e+e)%e+17,668265263)^n;return d=Math.imul(d^d>>>13,1274126177),((d^d>>>16)>>>0)/4294967295},u=h(s,r),p=h(s+1,r),f=h(s,r+1),g=h(s+1,r+1);return(u*(1-c)+p*c)*(1-l)+(f*(1-c)+g*c)*l}function G0(i=512){let t=new Uint8Array(i*i*4),e=new Float32Array(i*i);for(let s=0;s<i;s++)for(let r=0;r<i;r++){let o=0,a=.55;for(let c=0;c<5;c++){let l=5<<c;o+=V0(r/i*l,s/i*l,l,900+c)*a,a*=.52}e[s*i+r]=o}for(let s=0;s<i;s++)for(let r=0;r<i;r++){let o=e[s*i+(r-1+i)%i],a=e[s*i+(r+1)%i],c=e[(s-1+i)%i*i+r],l=e[(s+1)%i*i+r],h=(o-a)*2.8,u=(c-l)*2.8,p=1,f=Math.hypot(h,u,p);h/=f,u/=f,p/=f;let g=(s*i+r)*4;t[g]=Math.round((h*.5+.5)*255),t[g+1]=Math.round((u*.5+.5)*255),t[g+2]=Math.round((p*.5+.5)*255),t[g+3]=255}let n=new Ri(t,i,i,dn);return n.wrapS=n.wrapT=Un,n.magFilter=je,n.minFilter=qn,n.generateMipmaps=!0,n.needsUpdate=!0,n}function W0(){let t=new Float32Array(400);for(let l=0;l<400;l++){let h=l/400*Math.PI*2,u=55,p=360;for(let f=0;f<26;f++){let g=(u+p)/2;_e(Math.cos(h)*g,Math.sin(h)*g)>.06?u=g:p=g}t[l]=u}let e=401,n=new Float32Array(e*2*3),s=new Float32Array(e*2*2),r=[];for(let l=0;l<=400;l++){let h=l%400,u=h/400*Math.PI*2,p=Math.cos(u),f=Math.sin(u),g=t[h]-2.6,y=t[h]+1.1,m=l*2;if(n[m*3]=p*g,n[m*3+1]=.2,n[m*3+2]=f*g,n[(m+1)*3]=p*y,n[(m+1)*3+1]=.24,n[(m+1)*3+2]=f*y,s[m*2]=l/400,s[m*2+1]=0,s[(m+1)*2]=l/400,s[(m+1)*2+1]=1,l<400){let d=l*2;r.push(d,d+1,d+2,d+1,d+3,d+2)}}let o=new Ge;o.setAttribute("position",new ve(n,3)),o.setAttribute("uv",new ve(s,2)),o.setIndex(r);let a=new Pe({uniforms:{uTime:{value:0}},vertexShader:`
      varying vec2 vUv;
      void main() {
        vUv = uv;
        gl_Position = projectionMatrix * modelViewMatrix * vec4(position, 1.0);
      }`,fragmentShader:`
      uniform float uTime;
      varying vec2 vUv;
      void main() {
        float wave = sin(vUv.x * 62.0 + uTime * 1.8) * 0.5 + 0.5;
        float spark = sin(vUv.x * 23.0 - uTime * 3.2 + sin(vUv.x * 97.0) * 2.0) * 0.5 + 0.5;
        float a = (0.3 + 0.34 * wave * spark) * (1.0 - vUv.y * 0.75);
        if (a < 0.02) discard;
        gl_FragColor = vec4(0.94, 0.97, 0.92, a);
      }`,transparent:!0,depthWrite:!1,side:en,fog:!1}),c=new se(o,a);return c.renderOrder=6,c.frustumCulled=!1,c}function X0(){let i=document.createElement("canvas");i.width=i.height=128;let t=i.getContext("2d"),e=t.createRadialGradient(64,64,0,64,64,64);e.addColorStop(0,"rgba(230,240,255,1)"),e.addColorStop(.18,"rgba(200,220,255,0.55)"),e.addColorStop(.5,"rgba(140,170,230,0.12)"),e.addColorStop(1,"rgba(80,110,180,0)"),t.fillStyle=e,t.fillRect(0,0,128,128);let n=new Fn(i);return n.colorSpace=we,n}function Th(i,t,e=16.4){let n=new Ms;n.scale.setScalar(9e3),n.renderOrder=-1e3,n.material.uniforms.turbidity.value=8,n.material.uniforms.rayleigh.value=2.2,n.material.uniforms.mieCoefficient.value=.005,n.material.uniforms.mieDirectionalG.value=.8,i.add(n);let s=new U,r=new gs(16777215,3);r.castShadow=!0,r.shadow.mapSize.set(2048,2048),r.shadow.camera.near=30,r.shadow.camera.far=1600,r.shadow.camera.left=-290,r.shadow.camera.right=290,r.shadow.camera.top=290,r.shadow.camera.bottom=-290,r.shadow.bias=-6e-4,r.shadow.normalBias=.6,r.target.position.set(0,0,0),i.add(r,r.target);let o=new gs(10335976,0);o.position.set(0,300,0),i.add(o);let a=new jr(12376319,2894360,.65);i.add(a);let c=new Qr(2240580,.16);i.add(c);let l=new gs(11455448,0);l.position.set(150,520,300),l.target.position.set(0,0,0),i.add(l,l.target);let h=1800,u=new Float32Array(h*3),p=Ne(99);for(let N=0;N<h;N++){let de=p()*Math.PI*2,Yt=Math.acos(1-p()*.92),qt=5600;u[N*3]=Math.sin(Yt)*Math.cos(de)*qt,u[N*3+1]=Math.cos(Yt)*qt,u[N*3+2]=Math.sin(Yt)*Math.sin(de)*qt}let f=new Ge;f.setAttribute("position",new ve(u,3));let g=new Gs({color:13624063,size:2,sizeAttenuation:!1,transparent:!0,opacity:0,depthWrite:!1,fog:!1}),y=new Yr(f,g);y.renderOrder=-970,i.add(y);let m=new ze,d=new se(new $e(28,20,16),new Tn({color:14674175,fog:!1,depthWrite:!1})),x=new Xr(new Vs({map:X0(),color:12375295,transparent:!0,opacity:.8,depthWrite:!1,depthTest:!0,blending:pi,fog:!1}));x.scale.setScalar(260),m.add(d,x),m.renderOrder=-960,i.add(m);let _=[H0()];i.add(..._);let v=G0(),C=new co(new wn(640,640,1,1),{textureWidth:256,textureHeight:256,waterNormals:v,sunDirection:new U(.3,.8,.2).normalize(),sunColor:16773853,waterColor:732209,distortionScale:3,fog:!0,alpha:1});C.material.fragmentShader=C.material.fragmentShader.replace("float rf0 = 0.3;","float rf0 = 0.035;").replace("vec3( 0.1 )","vec3( 0.008 )"),C.material.needsUpdate=!0,C.rotation.x=-Math.PI/2,C.position.y=.06,C.material.uniforms.size.value=.2,i.add(C);let w=W0();i.add(w),i.fog=new fs(9611704,36e-5);let A=new _t(10467524),P=new _t(12157027),E=new _t(330773),M=new _t(16774368),R=new _t(16751181),F=new _t(11062527),V=new _t(16756858),z=new _t(2505318),k=new _t(16250090),I=new _t(16760200),Y=new _t(1779520),B=new _t(9215147),Q=new _t(6639710),ut=new _t(528670),Et=new _t(865338),zt=new _t(2107693),$t=new _t(199442),H={hours:e,sunDir:s,sunLight:r,moonLight:o,hemi:a,ambient:c,sky:n,water:C,stars:y,moonGroup:m,clouds:_,fogColor:new _t,sunElevation:0,isNight:!1},st=new _t,pt=new _t,ct=new _t,Pt=new U;function At(N){H.hours=(N%24+24)%24;let de=(H.hours-6)/12*Math.PI;s.set(Math.cos(de),Math.sin(de),.36).normalize();let Yt=s.y;H.sunElevation=Yt;let qt=Math.max(0,1-Math.abs(Yt)/.22),Vt=Math.max(0,Math.min(1,(Yt+.04)/.42)),Lt=Vt*Vt*(3-2*Vt),Ut=1-Math.max(0,Math.min(1,(Yt+.08)/.14));H.isNight=Yt<-.04,n.material.uniforms.sunPosition.value.copy(s),n.material.uniforms.turbidity.value=3.4+7.5*(1-Lt),n.material.uniforms.rayleigh.value=1.15+2.1*(1-Lt),n.material.uniforms.mieCoefficient.value=.0028+.0045*(1-Lt),r.position.copy(s).multiplyScalar(700),r.color.copy(R).lerp(M,$s.smoothstep(Yt,.02,.6)),r.intensity=.03+2.45*$s.smoothstep(Yt,-.05,.45),r.shadow.intensity=.7+.2*Lt,o.position.copy(s).multiplyScalar(-700),o.position.y=Math.max(120,o.position.y),o.intensity=.18*Ut,a.color.copy(z).lerp(V,qt).lerp(F,Lt),a.groundColor.setHex(3354653),a.intensity=.2+.66*Lt+.1*qt,c.intensity=.09+.12*Lt,l.intensity=.08+.62*Lt,l.color.copy(z).lerp(V,qt).lerp(F,Lt),H.fogColor.copy(E).lerp(P,qt*.8).lerp(A,Lt*.9),i.fog.color.copy(H.fogColor),i.fog.density=62e-5-22e-5*Lt,C.material.uniforms.sunDirection.value.copy(s),C.material.uniforms.sunColor.value.copy(R).lerp(M,Lt),ct.copy($t).lerp(zt,qt).lerp(Et,Lt),C.material.uniforms.waterColor.value.copy(ct),g.opacity=Math.pow(Ut,1.6)*.85,y.visible=g.opacity>.01,Pt.copy(s).multiplyScalar(-1),Pt.y=Math.abs(Pt.y)*.9+.15,Pt.normalize(),m.position.copy(Pt).multiplyScalar(4600),m.lookAt(0,0,0),m.visible=Ut>.05;for(let T=0;T<_.length;T++){let S=_[T].material;S.uniforms.uSun.value=.35+.65*Lt,st.copy(Y).lerp(I,qt).lerp(k,Lt),pt.copy(ut).lerp(Q,qt).lerp(B,Lt),S.uniforms.uLit.value.copy(st),S.uniforms.uShadow.value.copy(pt),S.uniforms.uSunDir.value.copy(s),S.uniforms.uOpacity.value=.06+.16*Lt+.04*qt,S.uniforms.uCoverage.value=.5+.05*(1-Lt)}i.environmentIntensity=.06+.32*Lt,t.toneMappingExposure=.63+.18*Lt}At(e);let Ot=new ps,Xt=new Ms;Xt.scale.setScalar(800),Xt.material.uniforms.turbidity.value=n.material.uniforms.turbidity.value,Xt.material.uniforms.rayleigh.value=n.material.uniforms.rayleigh.value,Xt.material.uniforms.mieCoefficient.value=n.material.uniforms.mieCoefficient.value,Xt.material.uniforms.mieDirectionalG.value=.8,Xt.material.uniforms.sunPosition.value.copy(s),Ot.add(Xt);let Jt=new ds(t),le=Jt.fromScene(Ot,0,.1,1200);return i.environment=le.texture,i.environmentIntensity=.55,Jt.dispose(),H.setTime=At,H.update=(N,de)=>{C.material.uniforms.time.value+=N*.55;for(let Yt of _)Yt.material.uniforms.uTime.value=de;y.rotation.y=de*.002,w.material.uniforms.uTime.value=de},H}function Fe(i,t=!1){let e=i[0].index!==null,n=new Set(Object.keys(i[0].attributes)),s=new Set(Object.keys(i[0].morphAttributes)),r={},o={},a=i[0].morphTargetsRelative,c=new Ge,l=0;for(let h=0;h<i.length;++h){let u=i[h],p=0;if(e!==(u.index!==null))return console.error("THREE.BufferGeometryUtils: .mergeGeometries() failed with geometry at index "+h+". All geometries must have compatible attributes; make sure index attribute exists among all geometries, or in none of them."),null;for(let f in u.attributes){if(!n.has(f))return console.error("THREE.BufferGeometryUtils: .mergeGeometries() failed with geometry at index "+h+'. All geometries must have compatible attributes; make sure "'+f+'" attribute exists among all geometries, or in none of them.'),null;r[f]===void 0&&(r[f]=[]),r[f].push(u.attributes[f]),p++}if(p!==n.size)return console.error("THREE.BufferGeometryUtils: .mergeGeometries() failed with geometry at index "+h+". Make sure all geometries have the same number of attributes."),null;if(a!==u.morphTargetsRelative)return console.error("THREE.BufferGeometryUtils: .mergeGeometries() failed with geometry at index "+h+". .morphTargetsRelative must be consistent throughout all geometries."),null;for(let f in u.morphAttributes){if(!s.has(f))return console.error("THREE.BufferGeometryUtils: .mergeGeometries() failed with geometry at index "+h+".  .morphAttributes must be consistent throughout all geometries."),null;o[f]===void 0&&(o[f]=[]),o[f].push(u.morphAttributes[f])}if(t){let f;if(e)f=u.index.count;else if(u.attributes.position!==void 0)f=u.attributes.position.count;else return console.error("THREE.BufferGeometryUtils: .mergeGeometries() failed with geometry at index "+h+". The geometry must have either an index or a position attribute"),null;c.addGroup(l,f,h),l+=f}}if(e){let h=0,u=[];for(let p=0;p<i.length;++p){let f=i[p].index;for(let g=0;g<f.count;++g)u.push(f.getX(g)+h);h+=i[p].attributes.position.count}c.setIndex(u)}for(let h in r){let u=wh(r[h]);if(!u)return console.error("THREE.BufferGeometryUtils: .mergeGeometries() failed while trying to merge the "+h+" attribute."),null;c.setAttribute(h,u)}for(let h in o){let u=o[h][0].length;if(u===0)break;c.morphAttributes=c.morphAttributes||{},c.morphAttributes[h]=[];for(let p=0;p<u;++p){let f=[];for(let y=0;y<o[h].length;++y)f.push(o[h][y][p]);let g=wh(f);if(!g)return console.error("THREE.BufferGeometryUtils: .mergeGeometries() failed while trying to merge the "+h+" morphAttribute."),null;c.morphAttributes[h].push(g)}}return c}function wh(i){let t,e,n,s=-1,r=0;for(let l=0;l<i.length;++l){let h=i[l];if(t===void 0&&(t=h.array.constructor),t!==h.array.constructor)return console.error("THREE.BufferGeometryUtils: .mergeAttributes() failed. BufferAttribute.array must be of consistent array types across matching attributes."),null;if(e===void 0&&(e=h.itemSize),e!==h.itemSize)return console.error("THREE.BufferGeometryUtils: .mergeAttributes() failed. BufferAttribute.itemSize must be consistent across matching attributes."),null;if(n===void 0&&(n=h.normalized),n!==h.normalized)return console.error("THREE.BufferGeometryUtils: .mergeAttributes() failed. BufferAttribute.normalized must be consistent across matching attributes."),null;if(s===-1&&(s=h.gpuType),s!==h.gpuType)return console.error("THREE.BufferGeometryUtils: .mergeAttributes() failed. BufferAttribute.gpuType must be consistent across matching attributes."),null;r+=h.count*e}let o=new t(r),a=new ve(o,e,n),c=0;for(let l=0;l<i.length;++l){let h=i[l];if(h.isInterleavedBufferAttribute){let u=c/e;for(let p=0,f=h.count;p<f;p++)for(let g=0;g<e;g++){let y=h.getComponent(p,g);a.setComponent(p+u,g,y)}}else o.set(h.array,c);c+=h.count*e}return s!==void 0&&(a.gpuType=s),a}function Ch(i,t,e,n){let s=Math.floor(i),r=Math.floor(t),o=i-s,a=t-r,c=o*o*(3-2*o),l=a*a*(3-2*a),h=(y,m)=>{let d=Math.imul((y%e+e)%e+31,374761393)^Math.imul((m%e+e)%e+17,668265263)^n;return d=Math.imul(d^d>>>13,1274126177),((d^d>>>16)>>>0)/4294967295},u=h(s,r),p=h(s+1,r),f=h(s,r+1),g=h(s+1,r+1);return(u*(1-c)+p*c)*(1-l)+(f*(1-c)+g*c)*l}function Y0(){let t=document.createElement("canvas");t.width=t.height=512;let e=t.getContext("2d"),n=document.createElement("canvas");n.width=n.height=512;let s=n.getContext("2d");e.fillStyle="#b9c0a5",e.fillRect(0,0,512,512),s.fillStyle="#808080",s.fillRect(0,0,512,512);let r=Ne(1177),o=(x,_,v)=>{let C=[x],w=[_];x<24&&C.push(x+512),x>488&&C.push(x-512),_<24&&w.push(_+512),_>488&&w.push(_-512);for(let A of C)for(let P of w)v(A,P)},a=["rgba(94,118,76,0.12)","rgba(186,186,148,0.12)","rgba(78,100,64,0.09)","rgba(206,202,170,0.09)"];for(let x=0;x<90;x++){let _=r()*512,v=r()*512,C=24+r()*90,w=a[Math.floor(r()*a.length)];o(_,v,(A,P)=>{let E=e.createRadialGradient(A,P,0,A,P,C);E.addColorStop(0,w),E.addColorStop(1,"rgba(0,0,0,0)"),e.fillStyle=E,e.fillRect(A-C,P-C,C*2,C*2)})}let c=[],l=[[120,140,92],[148,164,112],[178,182,140],[104,126,82],[170,178,128],[202,200,168],[142,135,102],[118,138,105]];for(let x=0;x<7200;x++){let _=l[Math.floor(r()*l.length)];c.push({x:r()*512,y:r()*512,len:3+r()*15,ang:-Math.PI/2+(r()-.5)*1.1,bend:(r()-.5)*4,w:.6+r()*1.7,a:.28+r()*.5,c:_})}for(let x of c)o(x.x,x.y,(_,v)=>{let C=_+Math.cos(x.ang)*x.len+x.bend,w=v+Math.sin(x.ang)*x.len;e.strokeStyle=`rgba(${x.c[0]},${x.c[1]},${x.c[2]},${x.a})`,e.lineWidth=x.w,e.beginPath(),e.moveTo(_,v),e.quadraticCurveTo(_+Math.cos(x.ang)*x.len*.55,v+Math.sin(x.ang)*x.len*.55,C,w),e.stroke();let A=128+48*x.a;s.strokeStyle=`rgb(${A},${A},${A})`,s.lineWidth=Math.max(.7,x.w),s.beginPath(),s.moveTo(_,v),s.quadraticCurveTo(_+Math.cos(x.ang)*x.len*.55,v+Math.sin(x.ang)*x.len*.55,C,w),s.stroke()});for(let x=0;x<900;x++){let _=r()*512,v=r()*512,C=.4+r()*1.4,w=800+r()*60;o(_,v,(A,P)=>{e.fillStyle=`rgba(${w*.92},${w*.9},${w*.84},0.5)`,e.beginPath(),e.arc(A,P,C,0,Math.PI*2),e.fill()})}let h=e.getImageData(0,0,512,512),u=s.getImageData(0,0,512,512),p=new Float32Array(512*512);for(let x=0;x<512*512;x++)p[x]=(u.data[x*4]+u.data[x*4+1]+u.data[x*4+2])/765;let f=document.createElement("canvas");f.width=f.height=512;let g=f.getContext("2d"),y=g.createImageData(512,512);for(let x=0;x<512;x++)for(let _=0;_<512;_++){let v=(x*512+_)*4,C=p[x*512+(_-1+512)%512],w=p[x*512+(_+1)%512],A=p[(x-1+512)%512*512+_],P=p[(x+1)%512*512+_],E=(C-w)*4.2,M=(A-P)*4.2,R=Math.hypot(E,M,1);E/=R,M/=R,y.data[v]=Math.round((E*.5+.5)*255),y.data[v+1]=Math.round((M*.5+.5)*255),y.data[v+2]=Math.round((.5+.5/R)*255),y.data[v+3]=255}g.putImageData(y,0,0);let m=new Fn(t);m.colorSpace=we,m.wrapS=m.wrapT=Un,m.repeat.set(118,118),m.anisotropy=8;let d=new Fn(f);return d.wrapS=d.wrapT=Un,d.repeat.set(168,168),d.anisotropy=8,{albedo:m,normal:d}}function q0(){let i=document.createElement("canvas");i.width=128,i.height=128;let t=i.getContext("2d");t.clearRect(0,0,128,128);let e=Ne(2024);for(let s=0;s<26;s++){let r=10+e()*108,o=52+e()*72,a=(e()-.5)*40,c=1.5+e()*3.2,l=t.createLinearGradient(0,128,0,128-o);l.addColorStop(0,"rgba(150,164,104,0.98)"),l.addColorStop(.55,"rgba(214,224,170,1)"),l.addColorStop(1,"rgba(248,250,224,1)"),t.strokeStyle=l,t.lineWidth=c,t.lineCap="round",t.beginPath(),t.moveTo(r,128),t.quadraticCurveTo(r+a*.2,128-o*.55,r+a,128-o),t.stroke()}let n=new Fn(i);return n.colorSpace=we,n.anisotropy=2,n}function Rh(i,t){let e=t.terrainSegments,n=new wn(1600,1600,e,e);n.rotateX(-Math.PI/2);let s=n.attributes.position;for(let Z=0;Z<s.count;Z++){let it=s.getX(Z),ft=s.getZ(Z);s.setY(Z,_e(it,ft))}n.computeVertexNormals();let r=n.attributes.normal,o=new Float32Array(s.count*3),a=new _t(4415783),c=new _t(3097117),l=new _t(7303230),h=new _t(6056754),u=new _t(11969918),p=new _t(8550753),f=new _t(7630435),g=new _t(4801854),y=new _t(14278369),m=new _t(9143674),d=new _t,x=new _t;for(let Z=0;Z<s.count;Z++){let it=s.getX(Z),ft=s.getZ(Z),rt=s.getY(Z),mt=Math.max(.06,r.getY(Z)),yt=Math.sqrt(1-mt*mt)/mt,Tt=Ch((it+800)*.006,(ft+800)*.006,64,33),kt=Ch((it+800)*.05,(ft+800)*.05,32,71);if(rt<0+.65)d.copy(p).lerp(u,tn(0,0+1.6,rt));else if(yt>.66||rt>80&&yt>.32)d.copy(g).lerp(f,Tt),rt>105&&yt>.48&&d.lerp(m,kt*.5);else if(rt>128+Tt*16)d.copy(f).lerp(y,tn(130,155,rt)*(1-tn(.7,1.25,yt)*.6));else{let D=tn(360,140,Math.hypot(it,ft));d.copy(c).lerp(a,D*.75+Tt*.25),d.lerp(l,_i((1-D)*.55+(Tt-.5)*.45,0,.7)),d.lerp(h,kt*.18)}let b=.82+kt*.3-tn(.5,1.3,yt)*.16;d.multiplyScalar(b),o[Z*3]=d.r,o[Z*3+1]=d.g,o[Z*3+2]=d.b}n.setAttribute("color",new ve(o,3));let _=Y0(),v=new Ae({vertexColors:!0,map:_.albedo,normalMap:_.normal,normalScale:new Wt(.48,.48),roughness:.94,metalness:0}),C=new se(n,v);C.receiveShadow=!0,C.castShadow=!1,C.name="terrain",i.add(C);let w=Ne(4242),A=q0(),P=$0(),E=new Ae({map:A,alphaTest:.38,side:en,roughness:.85,metalness:0,color:16777215,vertexColors:!1}),M={value:0};E.onBeforeCompile=Z=>{Z.uniforms.uWindTime=M,Z.vertexShader=`uniform float uWindTime;
`+Z.vertexShader,Z.vertexShader=Z.vertexShader.replace("#include <begin_vertex>",`#include <begin_vertex>
      #ifdef USE_INSTANCING
        vec3 ip = vec3(instanceMatrix[3][0], instanceMatrix[3][1], instanceMatrix[3][2]);
        float sway = sin(uWindTime * 1.45 + ip.x * 0.055 + ip.z * 0.032);
        float gust = sin(uWindTime * 0.32 + ip.x * 0.014 - ip.z * 0.01) * 0.5 + 0.5;
        transformed.x += sway * uv.y * (0.12 + gust * 0.12);
        transformed.z += cos(uWindTime * 1.15 + ip.z * 0.06) * uv.y * 0.1;
      #endif`)};let R=t.grassCount,F=new An(P,E,R);F.receiveShadow=!0,F.castShadow=!1;let V=[8229461,9085279,7308617,9939308,7834704],z=[2993453,12886064,8034906],k=[1.4,.9,1.1],I=new Re,Y=new _t,B=new ce,Q=new U,ut=new qe,Et=new sn,zt=new U,$t=new Float32Array(R),H=new Float32Array(R),st=new Float32Array(R),pt=new Float32Array(R),ct=new Float32Array(R),Pt=new Float32Array(R),At=new Float32Array(R),Ot=0,Xt=0,Jt=232,le=292;for(;Ot<R&&Xt<R*10;){Xt++;let Z,it;if(Ot<R*.78){let b=118*Math.sqrt(w()),D=w()*Math.PI*2;Z=Jt+Math.cos(D)*b,it=le+Math.sin(D)*b}else{let b=730*Math.sqrt(w()),D=w()*Math.PI*2;Z=Math.cos(D)*b,it=Math.sin(D)*b}let ft=_e(Z,it);if(ft<0+.55||ft>82||xi(Z,it)>.7)continue;let rt=Zs(Z,it);if(rt<.08||w()>.3+rt*.9)continue;I.position.set(Z,ft-.04,it);let mt=w()*Math.PI*2;I.rotation.set(0,mt,0);let yt=.95+w()*1.3,Tt=.72+w()*.8,kt=.72+w()*.8;I.scale.set(Tt,yt,kt),I.updateMatrix(),F.setMatrixAt(Ot,I.matrix),Y.setHex(V[Math.floor(w()*V.length)]),Y.offsetHSL(0,0,(w()-.5)*.1),F.setColorAt(Ot,Y),$t[Ot]=Z,H[Ot]=ft-.04,st[Ot]=it,pt[Ot]=mt,ct[Ot]=Tt,Pt[Ot]=yt,At[Ot]=kt,Ot++}F.count=Ot,F.instanceMatrix.needsUpdate=!0,F.instanceColor&&(F.instanceColor.needsUpdate=!0),F.computeBoundingSphere(),i.add(F);function N(Z,it,ft,rt,mt){let Tt=1/(ft/it),kt=ft/2,b=!1;for(let D=0;D<Ot;D++){let G=Math.floor(($t[D]+kt)*Tt),W=Math.floor((st[D]+kt)*Tt);G<0?G=0:G>=it&&(G=it-1),W<0?W=0:W>=it&&(W=it-1);let O=W*it+G,ht=Z[O],vt=.12+.88*ht,Ct=1,Bt=0;if(rt){let Qt=$t[D]-rt.x,Dt=st[D]-rt.z,oe=Qt*Qt+Dt*Dt;if(oe<25){let ue=Math.min(1,(5-Math.sqrt(oe))/2.4);Ct=1-.82*ue,Bt=ue}}let jt=(.5+.5*vt)*Ct;if(B.compose(Q.set($t[D],H[D],st[D]),ut.setFromEuler(Et.set(0,pt[D],0)),zt.set(ct[D]*jt,Pt[D]*vt*Ct,At[D]*jt)),F.setMatrixAt(D,B),mt&&ht>.02){let Qt=mt[0][O]+mt[1][O]+mt[2][O];if(Qt>.02){let Dt=0,oe=0,ue=0;for(let ae=0;ae<3;ae++){let Ht=mt[ae][O]*k[ae],an=z[ae];Dt+=(an>>16&255)/255*Ht,oe+=(an>>8&255)/255*Ht,ue+=(an&255)/255*Ht}let me=(.25+.75*Math.min(1,Qt*1.5))*(1-.25*Bt),Oe=.92+w()*.16;Y.setRGB(_i(Dt*me*Oe,0,1),_i(oe*me*Oe,0,1),_i(ue*me*Oe,0,1)),F.setColorAt(D,Y),b=!0}}}F.instanceMatrix.needsUpdate=!0,b&&F.instanceColor&&(F.instanceColor.needsUpdate=!0)}let de=K0(),Yt=new Ae({vertexColors:!0,map:de.albedo,normalMap:de.normal,normalScale:new Wt(.4,.4),alphaTest:.34,side:en,roughness:.9,metalness:0}),qt=new Ae({vertexColors:!0,roughness:.92,metalness:0}),Vt=[{name:"broadleaf",...J0(),capacity:t.treeCapacity},{name:"conifer",...j0(),capacity:t.treeCapacity},{name:"birch",...Q0(),capacity:Math.max(30,t.treeCapacity>>1)}],Lt=[],Ut=[],T=Ne(808);for(let Z of Vt){let it=new An(Z.trunkGeo,qt,Z.capacity),ft=new An(Z.canopyGeo,Yt,Z.capacity);for(let rt of[it,ft])rt.castShadow=!0,rt.receiveShadow=!0,rt.count=0,rt.frustumCulled=!0,i.add(rt);it.userData.instances=[],Lt.push({trunk:it,canopy:ft,data:it.userData.instances})}function S(Z){let it=Z.s;if(Z.age<40)return it*(.3+Z.age/40*.35);if(Z.age<120)return it*(.65+(Z.age-40)/80*.35);if(Z.age<400)return it;let ft=Z.age-400;return it*(1-Math.min(.35,ft/1200))}function X(Z,it,ft){let rt=ft*ft;for(let mt of Lt)for(let yt of mt.data){let Tt=yt.x-Z,kt=yt.z-it;if(Tt*Tt+kt*kt<rt)return!0}return!1}function j(Z,it,ft=null,rt=null,mt=null){let yt=_e(Z,it);if(yt<0+1.6||yt>112||xi(Z,it)>.62)return!1;let Tt=rt;if(Tt==null){let Ct=T();yt>58?Tt=1:yt>38?Tt=Ct<.72?1:0:Tt=Ct<.74?0:Ct<.95?1:2}let kt=Lt[Tt];if(kt.data.length>=kt.trunk.instanceMatrix.count)return!1;let b=ft??1+T()*.95,D=.92+T()*.18,G=T()*Math.PI*2,W=mt??T()*700,O={x:Z,z:it,s:b,vy:D,rot:G,variant:Tt,age:W,maxAge:520+T()*380,seedT:4+T()*14,stage:W<40?0:W<120?1:W<400?2:3,_lastSt:0},ht=S(O);O._lastSt=ht,I.position.set(Z,yt-.15,it),I.rotation.set(0,G,0),I.scale.set(ht,ht*D,ht),I.updateMatrix();let vt=kt.data.length;kt.trunk.setMatrixAt(vt,I.matrix),kt.canopy.setMatrixAt(vt,I.matrix),Y.setHex(Tt===1?14215372:16777215),Y.offsetHSL(0,0,(T()-.5)*.06),kt.trunk.setColorAt(vt,Y),kt.canopy.setColorAt(vt,Y),kt.data.push(O),kt.trunk.count=kt.data.length,kt.canopy.count=kt.data.length;for(let Ct of[kt.trunk,kt.canopy])Ct.instanceMatrix.needsUpdate=!0,Ct.instanceColor&&(Ct.instanceColor.needsUpdate=!0),Ct.computeBoundingSphere();return Ut.push({x:Z,z:it}),!0}function at(Z,it){let ft=Z.data.length-1,rt=new ce,mt=new _t;it!==ft&&(Z.trunk.getMatrixAt(ft,rt),Z.trunk.getColorAt(ft,mt),Z.trunk.setMatrixAt(it,rt),Z.trunk.setColorAt(it,mt),Z.canopy.getMatrixAt(ft,rt),Z.canopy.getColorAt(ft,mt),Z.canopy.setMatrixAt(it,rt),Z.canopy.setColorAt(it,mt),Z.data[it]=Z.data[ft]),Z.data.pop(),Z.trunk.count=Z.data.length,Z.canopy.count=Z.data.length;for(let yt of[Z.trunk,Z.canopy])yt.instanceMatrix.needsUpdate=!0,yt.instanceColor&&(yt.instanceColor.needsUpdate=!0),yt.computeBoundingSphere()}function J(Z,it,ft=6){let rt=null,mt=ft*ft;for(let yt of Lt)for(let Tt=0;Tt<yt.data.length;Tt++){let kt=yt.data[Tt].x-Z,b=yt.data[Tt].z-it,D=kt*kt+b*b;D<mt&&(mt=D,rt={entry:yt,i:Tt})}return rt?(at(rt.entry,rt.i),!0):!1}function Ft(Z,it,ft){let rt=null,mt=ft*ft;for(let yt of Lt)for(let Tt=0;Tt<yt.data.length;Tt++){let kt=yt.data[Tt],b=kt.x-Z,D=kt.z-it,G=b*b+D*D;G<mt&&(mt=G,rt={entry:yt,i:Tt,life:kt})}return rt}function St(Z,it){let ft=0;for(let rt of Lt)for(let mt of rt.data){if(mt.stage<2)continue;let yt=mt.x-Z,Tt=mt.z-it,kt=yt*yt+Tt*Tt,b=mt.s*3.4;if(kt<b*b&&(ft+=(1-Math.sqrt(kt)/b)*.4),ft>=1)return 1}return Math.min(1,ft)}function Rt(Z){let it=[[281,249,26],[298,229,20],[262,270,22],[231,258,18]];for(let ft=0;ft<6;ft++){let rt=T()*Math.PI*2,mt=22+T()*70,yt=Z.x+Math.cos(rt)*mt,Tt=Z.z+Math.sin(rt)*mt,kt=_e(yt,Tt);if(kt<0+1.6||kt>112||xi(yt,Tt)>.62||Zs(yt,Tt)<.15)continue;let b=!1;for(let[D,G,W]of it){let O=yt-D,ht=Tt-G;if(O*O+ht*ht<W*W){b=!0;break}}if(!b&&!X(yt,Tt,6+Z.s*2.2)&&j(yt,Tt,null,null,0))return}}function re(Z){if(!(Z<=0))for(let it of Lt){let ft=it.data;for(let rt=0;rt<ft.length;rt++){let mt=ft[rt],yt=Zs(mt.x,mt.z),Tt=1-Math.min(1,St(mt.x,mt.z)*.85);if(mt.age+=Z*(.35+yt*.9)*Tt,mt.seedT-=Z,mt.stage=mt.age<40?0:mt.age<120?1:mt.age<400?2:3,mt.stage>=2&&mt.seedT<=0&&(mt.seedT=(22+T()*22)/(.5+yt*1.2),T()<.1&&Rt(mt)),mt.age>500){let b=(mt.age-500)/400;if(T()<b*Z*.12){at(it,rt),rt--;continue}}let kt=S(mt);Math.abs(kt-mt._lastSt)>.012&&(mt._lastSt=kt,I.position.set(mt.x,_e(mt.x,mt.z)-.15,mt.z),I.rotation.set(0,mt.rot,0),I.scale.set(kt,kt*mt.vy,kt),I.updateMatrix(),it.trunk.setMatrixAt(rt,I.matrix),it.canopy.setMatrixAt(rt,I.matrix),it.trunk.instanceMatrix.needsUpdate=!0,it.canopy.instanceMatrix.needsUpdate=!0)}}}let lt=0,K=0,Nt=t.treeCount;for(;lt<Nt&&K<Nt*40;){K++;let Z=310+T()*420,it=T()*Math.PI*2,ft=Math.cos(it)*Z,rt=Math.sin(it)*Z;Zs(ft,rt)<.08&&T()>.25||j(ft,rt)&&lt++}for(let Z=0;Z<18;Z++){let it=-2.9+T()*1.55,ft=360+T()*190,rt=Math.cos(it)*ft,mt=Math.sin(it)*ft,yt=3+Math.floor(T()*4);for(let Tt=0;Tt<yt;Tt++){let kt=rt+(T()-.5)*58,b=mt+(T()-.5)*58;j(kt,b,1.15+T()*.85,null,250+T()*150)&&lt++}}let Zt=[[185,278,1.45,0],[305,275,1.5,1],[150,245,1.3,2],[325,250,1.45,0],[120,290,1.25,0],[340,240,1.35,1],[95,320,1.2,2]];for(let[Z,it,ft,rt]of Zt)j(Z,it,ft,rt,250+T()*150)&&lt++;let It=t_(31),ne=new Ae({vertexColors:!0,roughness:.96,metalness:0,flatShading:!0}),Kt=120,Gt=new An(It,ne,Kt);Gt.castShadow=!0,Gt.receiveShadow=!0;let L=Ne(515),dt=[],q=0,tt=0;for(;q<Kt&&tt<Kt*12;){tt++;let Z=230+L()*560,it=L()*Math.PI*2,ft=Math.cos(it)*Z,rt=Math.sin(it)*Z,mt=_e(ft,rt),yt=xi(ft,rt);if(mt<0-.5||yt<.28&&L()>.18)continue;let Tt=.5+Math.pow(L(),2.2)*4.2;I.position.set(ft,mt+Tt*.12,rt),I.rotation.set(L()*.6,L()*Math.PI*2,L()*.6),I.scale.set(Tt*(.8+L()*.5),Tt*(.55+L()*.35),Tt*(.8+L()*.5)),I.updateMatrix(),Gt.setMatrixAt(q,I.matrix),Y.setHex(L()>.5?13682615:11511706),Gt.setColorAt(q,Y),dt.push({x:ft,z:rt}),q++}Gt.count=q,Gt.instanceMatrix.needsUpdate=!0,Gt.instanceColor&&(Gt.instanceColor.needsUpdate=!0),Gt.computeBoundingSphere(),i.add(Gt);function wt(Z,it){if(q>=Kt)return!1;let ft=_e(Z,it);if(ft<0-.4||xi(Z,it)>1.2)return!1;let rt=.5+Math.pow(L(),2.2)*4.2;return I.position.set(Z,ft+rt*.12,it),I.rotation.set(L()*.6,L()*Math.PI*2,L()*.6),I.scale.set(rt*(.8+L()*.5),rt*(.55+L()*.35),rt*(.8+L()*.5)),I.updateMatrix(),Gt.setMatrixAt(q,I.matrix),Y.setHex(L()>.5?13682615:11511706),Gt.setColorAt(q,Y),dt.push({x:Z,z:it}),q++,Gt.count=q,Gt.instanceMatrix.needsUpdate=!0,Gt.instanceColor&&(Gt.instanceColor.needsUpdate=!0),Gt.computeBoundingSphere(),!0}let bt=e_(),et=new Ae({vertexColors:!0,roughness:.95}),ot=t.bushCount,$=new An(bt,et,ot);$.castShadow=!0,$.receiveShadow=!0;let nt=Ne(616),gt=[],xt=0,te=0;for(;xt<ot-30&&te<ot*12;){te++;let Z=170+nt()*430,it=nt()*Math.PI*2,ft=Math.cos(it)*Z,rt=Math.sin(it)*Z,mt=_e(ft,rt);if(mt<0+1.5||mt>72||xi(ft,rt)>.55)continue;let yt=.55+nt()*1.3;I.position.set(ft,mt,rt),I.rotation.set(0,nt()*Math.PI*2,0),I.scale.set(yt,yt*(.7+nt()*.5),yt),I.updateMatrix(),$.setMatrixAt(xt,I.matrix),Y.setHex(nt()>.5?6388281:7572293),$.setColorAt(xt,Y),gt.push({x:ft,z:rt}),xt++}$.count=xt,$.instanceMatrix.needsUpdate=!0,$.instanceColor&&($.instanceColor.needsUpdate=!0),$.computeBoundingSphere(),i.add($);function He(Z){let it=q-1;Z!==it&&(Gt.getMatrixAt(it,I.matrix),Gt.setMatrixAt(Z,I.matrix),Gt.instanceColor&&(Gt.getColorAt(it,Y),Gt.setColorAt(Z,Y)),dt[Z]=dt[it]),dt.pop(),q--,Gt.count=q,Gt.instanceMatrix.needsUpdate=!0,Gt.instanceColor&&(Gt.instanceColor.needsUpdate=!0),Gt.computeBoundingSphere()}function Ze(Z,it){let ft=_e(Z,it);if(xt>=ot)return!1;let rt=.55+Math.random()*.6;return I.position.set(Z,ft,it),I.rotation.set(0,Math.random()*Math.PI*2,0),I.scale.set(rt,rt*.9,rt),I.updateMatrix(),$.setMatrixAt(xt,I.matrix),Y.setHex(Math.random()>.5?6388281:7572293),$.setColorAt(xt,Y),gt.push({x:Z,z:it}),xt++,$.count=xt,$.instanceMatrix.needsUpdate=!0,$.instanceColor&&($.instanceColor.needsUpdate=!0),$.computeBoundingSphere(),!0}function fe(Z){let it=xt-1;Z!==it&&($.getMatrixAt(it,I.matrix),$.setMatrixAt(Z,I.matrix),$.instanceColor&&($.getColorAt(it,Y),$.setColorAt(Z,Y)),gt[Z]=gt[it]),gt.pop(),xt--,$.count=xt,$.instanceMatrix.needsUpdate=!0,$.instanceColor&&($.instanceColor.needsUpdate=!0),$.computeBoundingSphere()}function zn(Z,it,ft){let rt=-1,mt=ft*ft;for(let yt=0;yt<dt.length;yt++){let Tt=dt[yt].x-Z,kt=dt[yt].z-it,b=Tt*Tt+kt*kt;b<mt&&(mt=b,rt=yt)}return rt>=0?{i:rt,x:dt[rt].x,z:dt[rt].z}:null}function kn(Z,it,ft){let rt=-1,mt=ft*ft;for(let yt=0;yt<gt.length;yt++){let Tt=gt[yt].x-Z,kt=gt[yt].z-it,b=Tt*Tt+kt*kt;b<mt&&(mt=b,rt=yt)}return rt>=0?{i:rt,x:gt[rt].x,z:gt[rt].z}:null}return{terrain:C,grass:F,grassMaterial:E,rocks:Gt,bushes:$,treeMeshes:Lt,treeData:Ut,addTree:j,removeTree:J,nearestTree:Ft,shadeAt:St,addRock:wt,nearestRock:zn,nearestBush:kn,addBush:Ze,removeRockAt:He,removeBushAt:fe,updateTrees:re,updateGrassVisual:N,treeCount:()=>Lt.reduce((Z,it)=>Z+it.data.length,0),update:Z=>{M.value=Z}}}function $0(){let i=[];for(let t=0;t<2;t++){let e=new wn(1,1,1,1);e.translate(0,.5,0),e.rotateY(t/2*Math.PI),i.push(e)}return Fe(i,!1)}function Z0(i,t,e=.05,n=1){let s=i.attributes.position.count,r=new Float32Array(s*3),o=new _t(t),a=Ne(n);for(let c=0;c<s;c++){let l=1+(a()-.5)*e*2;r[c*3]=o.r*l,r[c*3+1]=o.g*l,r[c*3+2]=o.b*l}return i.setAttribute("color",new ve(r,3)),i}function ei(i,t,e,n,s){Z0(t,e,n,s),i.push(t)}function Ph(i,t,e,n,s,r){let o=new ce,a=new qe().setFromEuler(new sn(0,n,s,"YXZ")),c=new U(i,t,e).add(new U(0,r/2,0).applyQuaternion(a));return o.compose(c,a,new U(1,1,1)),o}function K0(){let t=Ne(909),e=[];for(let g=0;g<64;g++){let y=t()*256,m=t()*256,d=2+Math.floor(t()*5);for(let x=0;x<d;x++)e.push({x:y+(t()-.5)*24,y:m+(t()-.5)*18,rw:3+t()*9,rh:2.5+t()*7,ang:t()*Math.PI,shade:.5+t()*.5})}let n=document.createElement("canvas");n.width=n.height=256;let s=n.getContext("2d"),r=document.createElement("canvas");r.width=r.height=256;let o=r.getContext("2d");s.clearRect(0,0,256,256),o.fillStyle="#000",o.fillRect(0,0,256,256);for(let g of e){let y=Math.min(255,Math.round(96*g.shade+46)),m=Math.min(255,Math.round(y*1.06)),d=Math.min(255,Math.round(y*.7)),x=[g.x],_=[g.y];g.x<20&&x.push(g.x+256),g.x>236&&x.push(g.x-256),g.y<20&&_.push(g.y+256),g.y>236&&_.push(g.y-256);for(let v of x)for(let C of _)s.save(),s.translate(v,C),s.rotate(g.ang),s.fillStyle=`rgba(${m},${y},${d},0.97)`,s.beginPath(),s.ellipse(0,0,g.rw,g.rh,0,0,Math.PI*2),s.fill(),s.restore(),o.save(),o.translate(v,C),o.rotate(g.ang),o.fillStyle="rgba(190,190,190,0.9)",o.beginPath(),o.ellipse(0,0,g.rw,g.rh,0,0,Math.PI*2),o.fill(),o.restore()}let a=o.getImageData(0,0,256,256),c=new Float32Array(256*256);for(let g=0;g<256*256;g++)c[g]=(a.data[g*4]+a.data[g*4+1]+a.data[g*4+2])/765;let l=document.createElement("canvas");l.width=l.height=256;let h=l.getContext("2d"),u=h.createImageData(256,256);for(let g=0;g<256;g++)for(let y=0;y<256;y++){let m=(g*256+y)*4,d=c[g*256+(y-1+256)%256],x=c[g*256+(y+1)%256],_=c[(g-1+256)%256*256+y],v=c[(g+1)%256*256+y],C=(d-x)*5,w=(_-v)*5,A=Math.hypot(C,w,1);C/=A,w/=A,u.data[m]=Math.round((C*.5+.5)*255),u.data[m+1]=Math.round((w*.5+.5)*255),u.data[m+2]=Math.round((.5+.5/A)*255),u.data[m+3]=255}h.putImageData(u,0,0);let p=new Fn(n);p.colorSpace=we,p.wrapS=p.wrapT=Un;let f=new Fn(l);return f.wrapS=f.wrapT=Un,{albedo:p,normal:f}}function J0(){let i=[],t=[],e=11,n=new De(.32,.55,7.2,7,1);n.translate(0,3.6,0),ei(i,n,5916209,.13,e++);for(let o=0;o<4;o++){let a=new De(.09,.2,3.4,5,1);a.applyMatrix4(Ph(0,3.5+o*.6,0,o*1.7,.55+o%2*.2,3.4)),ei(i,a,5390123,.1,e++)}let s=[3821606,4612140,5270322,3229727,5927992],r=[[0,7.6,0,3.8,2.5,3.8],[2.6,7.3,.8,2.8,2.1,2.8],[-2.4,7.5,-.9,2.9,2,2.7],[.9,9.2,-2,2.5,2,2.6],[-1.5,9,1.8,2.4,1.9,2.5],[1.8,9.1,2.1,2.1,1.8,2.2],[-.4,10.6,.3,2,1.7,2],[2.9,8.2,-1.5,1.8,1.6,1.8],[-2.9,8.4,1.1,1.7,1.5,1.7],[0,6.7,-2.4,2.2,1.7,2.1]];for(let o=0;o<r.length;o++){let[a,c,l,h,u,p]=r[o],f=new $e(1,8,6);f.scale(h,u,p),f.translate(a,c,l),ei(t,f,s[o%s.length],.16,e++)}return{trunkGeo:Fe(i,!1),canopyGeo:Fe(t,!1)}}function j0(){let i=[],t=[],e=41,n=new De(.25,.48,12.5,7,1);n.translate(0,6.25,0),ei(i,n,4864553,.12,e++);let s=[[3.8,4.4,4.1],[3.2,4,6.1],[2.5,3.7,8],[1.8,3.4,9.8],[1.1,3,11.4]],r=[2177054,2703908,3165481];for(let o=0;o<s.length;o++){let[a,c,l]=s[o],h=new Cn(a,c,9,1);h.translate(0,l,0),ei(t,h,r[o%3],.15,e++)}return{trunkGeo:Fe(i,!1),canopyGeo:Fe(t,!1)}}function Q0(){let i=[],t=[],e=71,n=new De(.14,.28,7.8,6,1);n.translate(0,3.9,0),ei(i,n,9735030,.1,e++);for(let o=0;o<3;o++){let a=new De(.05,.11,2.8,4,1);a.applyMatrix4(Ph(0,4.2+o*.6,0,o*2.1,.62,2.8)),ei(i,a,9077360,.1,e++)}let s=[[0,7.4,0,2.4,2,2.3],[1.5,7.1,.5,1.9,1.5,1.8],[-1.4,7.2,-.4,1.8,1.4,1.7],[.3,8.7,-.4,1.7,1.5,1.6],[-.4,8.8,.9,1.5,1.2,1.4]],r=[6716470,7572031,5795885];for(let o=0;o<s.length;o++){let[a,c,l,h,u,p]=s[o],f=new $e(1,8,6);f.scale(h,u,p),f.translate(a,c,l),ei(t,f,r[o%3],.18,e++)}return{trunkGeo:Fe(i,!1),canopyGeo:Fe(t,!1)}}function t_(i){let t=new $r(1,1),e=t.attributes.position,n=Ne(i),s=new Map;for(let l=0;l<e.count;l++){let h=e.getX(l),u=e.getY(l),p=e.getZ(l),f=`${h.toFixed(3)},${u.toFixed(3)},${p.toFixed(3)}`,g=s.get(f);g===void 0&&(g=.78+n()*.4,s.set(f,g)),e.setXYZ(l,h*g,u*g*.72,p*g)}t.computeVertexNormals();let r=e.count,o=new Float32Array(r*3),a=new _t(16777215),c=Ne(i+1);for(let l=0;l<r;l++){let h=.62+c()*.35;o[l*3]=a.r*h,o[l*3+1]=a.g*h,o[l*3+2]=a.b*h}return t.setAttribute("color",new ve(o,3)),t}function e_(){let i=[],t=91,e=[[0,.75,0,1.1,.85,1],[.75,.6,.2,.8,.68,.75],[-.65,.55,-.2,.85,.6,.8],[.1,1.35,.1,.7,.6,.7]],n=[4018722,4940073,5861169];for(let s=0;s<e.length;s++){let[r,o,a,c,l,h]=e[s],u=new $e(1,6,4);u.scale(c,l,h),u.translate(r,o,a),ei(i,u,n[s%3],.18,t++)}return Fe(i,!1)}function n_(i,t,e=.06,n=1){let s=i.attributes.position.count,r=new Float32Array(s*3),o=new _t(t),a=Ne(n);for(let c=0;c<s;c++){let l=1+(a()-.5)*e*2;r[c*3]=o.r*l,r[c*3+1]=o.g*l,r[c*3+2]=o.b*l}return i.setAttribute("color",new ve(r,3)),i}function on(i,t,e,n=.05,s=1){n_(t,e,n,s),i.push(t)}function Me(i,t,e,n,s,r,o,a,c,l=0){let h=new Qe(t,e,n);l&&h.rotateY(l),h.translate(s,r,o),on(i,h,a,.04,c)}function i_(){let i=[],t=21;Me(i,8.6,.7,8.6,0,.35,0,10130309,t++),Me(i,7.6,4.4,7.6,0,2.8,0,15787988,t++);for(let u of[-1,1])for(let p of[-1,1])Me(i,.32,4.5,.32,u*3.7,2.8,p*3.7,9071176,t++);Me(i,1.5,2.4,.16,0,1.65,3.86,7031342,t++),Me(i,1.9,.26,1.2,0,.55,4.2,12167303,t++);let e=new Qe(5.4,.42,8.9);e.rotateZ(.345),e.translate(-2.35,5.5,0),on(i,e,4157327,.04,t++);let n=new Qe(5.4,.42,8.9);n.rotateZ(-.345),n.translate(2.35,5.5,0),on(i,n,4157327,.04,t++),Me(i,.5,.34,9,0,6.28,0,3102320,t++),Me(i,.9,2.3,.9,2.3,6.6,2.1,10321244,t++),Me(i,1,.3,1,2.3,7.75,2.1,7297346,t++);let s=Fe(i,!1),r=new Ae({vertexColors:!0,roughness:.86,metalness:0}),o=new se(s,r);o.castShadow=!0,o.receiveShadow=!0;let a=new ze;a.add(o);let c=new Tn({color:16761963}),l=new Qe(1.25,1.25,.09);for(let u of[-2.05,2.05]){let p=new se(l,c);p.position.set(u,3.05,3.86),a.add(p)}let h=new se(new Qe(.09,1.2,1.2),c);return h.position.set(3.86,3.05,-1.4),a.add(h),a}function Ih(i){let t=Math.cos(i),e=Math.sin(i),n=70,s=360;for(let r=0;r<20;r++){let o=(n+s)/2;_e(t*o,e*o)>.55?n=o:s=o}return{x:t*s,z:e*s,r:s,ca:t,sa:e}}function s_(i){let t=[],e=61,n=22,s=1.7;for(let g=0;g<13;g++)Me(t,3.5,.16,1.25,0,.56,1+g*1.62,g%2?9925711:9070663,e++);Me(t,.34,.34,n,-1.35,.42,n/2,7163700,e++),Me(t,.34,.34,n,1.35,.42,n/2,7163700,e++);for(let g of[3,9,15,21])for(let y of[-1.45,1.45])Me(t,.3,2.9,.3,y,-.9,g,6178096,e++);let r=Fe(t,!1),o=new Ae({vertexColors:!0,roughness:.92,metalness:0}),a=new se(r,o);a.castShadow=!0,a.receiveShadow=!0;let c=new ze;c.add(a);let l=[],h=71;Me(l,1.5,.55,3.8,0,.35,0,8015664,h++),Me(l,1.85,.22,4.1,0,.66,0,11568730,h++),Me(l,1.1,.12,1.2,0,.62,.9,6176037,h++),Me(l,1.1,.12,1.2,0,.62,-.9,6176037,h++);let u=new se(Fe(l,!1),new Ae({vertexColors:!0,roughness:.88,metalness:0}));u.castShadow=!0,u.position.set(3.1,.05,12),u.rotation.y=.08,c.add(u);let p=Ih(i),f=Math.atan2(-p.ca,-p.sa);return c.position.set(p.x-p.ca*2,0,p.z-p.sa*2),c.rotation.y=f,c}function r_(i,t,e=12){let n=[],s=91,r=Ne(s);for(let a=0;a<e;a++){let c=(a+.5)/e,l=i.x+(t.x-i.x)*c+(r()-.5)*2.2,h=i.z+(t.z-i.z)*c+(r()-.5)*2.2,u=_e(l,h);if(u<.5)continue;let p=new De(.7+r()*.35,.8+r()*.3,.18,7);p.translate(l,u+.08,h),on(n,p,r()>.5?11775131:10130308,.08,s++)}if(!n.length)return null;let o=new se(Fe(n,!1),new Ae({vertexColors:!0,roughness:.95}));return o.receiveShadow=!0,o}function o_(i){let t=Ne(2026),e=new Ae({color:16777215,roughness:1,metalness:0,flatShading:!0,emissive:9413819,emissiveIntensity:.18,fog:!1}),n=[];for(let s=0;s<14;s++){let r=[],o=3+Math.floor(t()*3);for(let u=0;u<o;u++){let p=new $e(1,7,5),f=3.5+t()*4,g=2.5+t()*2,y=3+t()*4;p.scale(f,g,y),p.translate((t()-.5)*18,(t()-.5)*4,(t()-.5)*14),r.push(p)}let a=Fe(r,!1),c=new se(a,e),l=-.8+t()*2.4,h=480+t()*320;c.position.set(Math.cos(l)*h,260+t()*110,Math.sin(l)*h),c.rotation.y=t()*Math.PI,c.castShadow=!1,c.receiveShadow=!1,c.renderOrder=4,i.add(c),n.push({mesh:c,speed:.5+t()*.8,phase:t()*Math.PI*2})}return n}function a_(){let i=[],t=111,e=new De(1.05,2.3,13,8);e.translate(0,6.5,0),on(i,e,15788761,.05,t++);let n=new De(2.6,2.85,1,8);n.translate(0,.5,0),on(i,n,13221800,.05,t++);let s=new Cn(1.55,2.4,8);s.translate(0,14.2,0),on(i,s,11618874,.05,t++);let r=new se(Fe(i,!1),new Ae({vertexColors:!0,roughness:.85,metalness:0}));r.castShadow=!0,r.receiveShadow=!0;let o=new ze;o.add(r);let a=new ze,c=new Ae({color:15919832,roughness:.8});for(let l=0;l<4;l++){let h=new se(new Qe(.55,5.4,.2),c);h.position.y=3;let u=new ze;u.add(h),u.rotation.z=l*Math.PI/2,a.add(u)}return a.position.set(0,12.9,1.15),o.add(a),{group:o,blades:a}}function c_(){let i=[],t=141,e=new De(6.5,6.5,.35,18);e.translate(0,.17,0),on(i,e,14731663,.06,t++);for(let r of[-2.6,2.6])Me(i,.32,3.4,.32,r,1.9,-2,9071176,t++);Me(i,6,.3,.3,0,3.55,-2,12999498,t++);for(let r of[-1.6,1.6])Me(i,.5,.22,.5,r,1.5,-2,4882357,t++);Me(i,1.7,.24,1.7,3.4,1.85,.6,11618874,t++);for(let r of[2.7,4.1])for(let o of[-.1,1.3])Me(i,.22,1.9,.22,r,.95,o,9071176,t++);let n=new Qe(1.5,.2,4.4);n.rotateX(.62),n.translate(3.4,.95,2.9),on(i,n,14264383,.05,t++),Me(i,.24,.24,2.2,3,.7,1.5,9071176,t++);let s=new se(Fe(i,!1),new Ae({vertexColors:!0,roughness:.85,metalness:0}));return s.castShadow=!0,s.receiveShadow=!0,s}function l_(){let i=[],t=201,e=new De(2.4,3,1.2,10);e.translate(0,.6,0),on(i,e,11775131,.05,t++);let n=6,s=1.6,r=1.95,o=.8;for(let f=0;f<n;f++){let g=f/n,y=(f+1)/n,m=new De(r+(o-r)*y,r+(o-r)*g,s,10);m.translate(0,1.2+s*(f+.5),0),on(i,m,f%2?15788761:12668476,.04,t++)}let a=new De(1.25,1.45,.5,10);a.translate(0,1.2+s*n+.25,0),on(i,a,9071176,.05,t++);let c=new De(.8,.9,1.25,10);c.translate(0,1.2+s*n+.5+.625,0),on(i,c,16117728,.03,t++);let l=new Cn(1,1,10);l.translate(0,1.2+s*n+.5+1.25+.5,0),on(i,l,11618874,.04,t++);let h=new se(Fe(i,!1),new Ae({vertexColors:!0,roughness:.85,metalness:0}));h.castShadow=!0,h.receiveShadow=!0;let u=new ze;u.add(h);let p=new se(new De(.66,.66,.85,10),new Tn({color:16771488}));return p.position.set(0,1.2+s*n+.5+.625,0),u.add(p),u}function h_(){let i=[[134,129,.6],[145,101,2.2],[106,123,4]],t=new ze;for(let[e,n,s]of i){let r=[],o=71;Me(r,1.5,.55,3.8,0,.35,0,8015664,o++),Me(r,1.85,.22,4.1,0,.66,0,11568730,o++),Me(r,1.1,.12,1.2,0,.62,.9,6176037,o++),Me(r,1.1,.12,1.2,0,.62,-.9,6176037,o++);let a=new se(Fe(r,!1),new Ae({vertexColors:!0,roughness:.88,metalness:0}));a.castShadow=!0,a.receiveShadow=!0;let c=new ze;c.add(a),c.position.set(e,.02,n),c.rotation.y=s,c.scale.setScalar(2),t.add(c)}return t}function Lh(i){let t=new ze,e=i_(),n=281,s=249,r=_e(n,s);e.position.set(n,r-.1,s),e.rotation.y=Math.atan2(-n,-s),e.scale.setScalar(1.5),t.add(e);let o=Math.atan2(s,n),a=s_(o);a.scale.setScalar(1.4),t.add(a);let c=Ih(o),l=r_({x:n,z:s+2},{x:c.x+c.ca*4,z:c.z+c.sa*4},13);l&&t.add(l);let h=a_(),u=262,p=270;h.group.position.set(u,_e(u,p)-.15,p),h.group.rotation.y=Math.atan2(-159-u,-136-p),h.group.scale.setScalar(1.3),t.add(h.group);let f=c_(),g=298,y=229;f.position.set(g,_e(g,y)-.1,y),f.rotation.y=Math.atan2(-159-g,-136-y),f.scale.setScalar(1.3),t.add(f);let m=l_(),d=231,x=258;m.position.set(d,_e(d,x)-.12,x),m.rotation.y=Math.atan2(-159-d,-136-x),m.scale.setScalar(1.3),t.add(m);let _=h_();t.add(_),i.add(t);let v=o_(i),C=Ne(777),w=Math.cos(3.86),A=Math.sin(3.86),P=A,E=-w,M=(V,z,k)=>new U(281+w*V+P*z,k,249+A*V+E*z),R=[M(-70,-90,48),M(-80,25,54),M(-90,95,60),M(-100,-45,66),M(-110,60,50),M(-120,-105,56),M(-130,15,62),M(-140,80,68),M(-150,-70,52),M(-160,40,58),M(-170,-25,64),M(-180,100,60),M(-190,-85,54),M(-200,55,70)],F=!1;return{group:t,clouds:v,setIso(V){F=V},update(V,z){h.blades.rotation.z=z*1;for(let k=0;k<v.length;k++){let I=v[k],Y=I.mesh.position;if(F){let B=R[k%R.length];Y.x=B.x,Y.y=B.y+Math.sin(z*.5+I.phase)*3,Y.z=B.z,I.mesh.rotation.y=z*.02}else{if(Y.x+=V*I.speed*1.5,Y.z+=V*I.speed*.35,Math.hypot(Y.x,Y.z)>820){let Q=Math.PI+C()*Math.PI;Y.x=Math.cos(Q)*780,Y.z=Math.sin(Q)*780}Y.y+=Math.sin(z*.15+I.phase)*V*.5}}}}}var Rc={terrainSegments:312,grassCount:36e3,treeCount:192,treeCapacity:260,bushCount:390,dpr:1.5};function Dh(i){let t=new Gr({canvas:i,antialias:!0,powerPreference:"high-performance",stencil:!1});t.setPixelRatio(Math.min(window.devicePixelRatio||1,Rc.dpr)),t.setSize(window.innerWidth,window.innerHeight,!1),t.shadowMap.enabled=!0,t.shadowMap.type=dc,t.toneMapping=pc,t.toneMappingExposure=1,t.outputColorSpace=we;let e=new ps;e.fog=new fs(724755,62e-5);let n=window.innerWidth/window.innerHeight<.85,s=new Ve(n?60:52,window.innerWidth/window.innerHeight,.4,22e3);s.position.set(270,_e(270,340)+43,340);let r=new ao(s,i);return r.target.set(-45,10,-110),r.enableDamping=!0,r.dampingFactor=.055,r.minDistance=7,r.maxDistance=980,r.minPolarAngle=.16,r.maxPolarAngle=Math.PI*.492,r.enablePan=!1,r.autoRotate=!0,r.autoRotateSpeed=.075,r.update(),window.addEventListener("resize",()=>{let o=window.innerWidth,a=window.innerHeight;s.aspect=o/a,s.fov=o/a<.85?60:52,s.updateProjectionMatrix(),t.setPixelRatio(Math.min(window.devicePixelRatio||1,Rc.dpr)),t.setSize(o,a,!1)}),{renderer:t,scene:e,camera:s,controls:r}}function Uh(i,t){let e=Th(i,t),n=Rh(i,Rc),s=Lh(i);return{env:e,terrain:n,landmarks:s}}function Nh(i,t){i.sunLight.target.position.set(t.x,0,t.z),i.sunLight.position.copy(i.sunDir).multiplyScalar(620),i.sunLight.position.x+=t.x,i.sunLight.position.z+=t.z,i.sunLight.position.y=Math.max(i.sunLight.position.y,15),i.sunLight.target.updateMatrixWorld()}var u_=`
  uniform sampler2D tMineral;
  uniform float uOpacity;
  varying vec2 vUv;
  void main() {
    float v = texture2D(tMineral, vUv).r;
    // piecewise color ramp
    vec3 cold = vec3(0.10, 0.18, 0.55);   // deep blue
    vec3 mid  = vec3(0.85, 0.72, 0.20);   // amber
    vec3 hot  = vec3(0.92, 0.20, 0.10);   // red-orange
    vec3 color = mix(cold, mid, smoothstep(0.0, 0.5, v));
    color = mix(color, hot, smoothstep(0.5, 1.0, v));
    float alpha = uOpacity * v;            // transparent where no mineral
    gl_FragColor = vec4(color, alpha);
  }
`,d_=`
  varying vec2 vUv;
  void main() {
    vUv = uv;
    gl_Position = projectionMatrix * modelViewMatrix * vec4(position, 1.0);
  }
`;function f_(i=38){let t=new Float32Array(i*i),e=new Ri(t,i,i,qs,_n);return e.magFilter=je,e.minFilter=je,e.wrapS=Ln,e.wrapT=Ln,e.needsUpdate=!0,{material:new Pe({uniforms:{tMineral:{value:e},uOpacity:{value:.65}},vertexShader:d_,fragmentShader:u_,transparent:!0,depthWrite:!1,blending:pi}),texture:e}}function Fh(i,t){if(!t||t.length===0)return!1;let e=i.image.data;if(e.length!==t.length)return console.warn("[mineral-mat] field length mismatch",e.length,t.length),!1;for(let n=0;n<e.length;n++)e[n]=t[n];return i.needsUpdate=!0,!0}function Oh(i=1080,t=38){let{material:e,texture:n}=f_(t),s=new wn(i,i),r=new se(s,e);return r.rotation.x=-Math.PI/2,r.position.y=.02,r.renderOrder=1,r.frustumCulled=!1,{mesh:r,material:e,texture:n}}function Pc(i,t,e){if(i=(i%1+1)%1,t<=0){let a=Math.round(e*255);return[a,a,a]}let n=e<.5?e*(1+t):e+t-e*t,s=2*e-n,r=a=>(a<0&&(a+=1),a>1&&(a-=1),a<1/6?s+(n-s)*6*a:a<.5?n:a<2/3?s+(n-s)*(2/3-a)*6:s),o=a=>Math.round(a*255);return[o(r(i+1/3)),o(r(i)),o(r(i-1/3))]}function Bh(i,t){return .629+(i+.5)/t*.242}function ho(i,t,e,n){i.beginPath(),i.moveTo(n,.5),i.lineTo(t-n,.5),i.quadraticCurveTo(t-.5,.5,t-.5,n),i.lineTo(t-.5,e-n),i.quadraticCurveTo(t-.5,e-.5,t-n,e-.5),i.lineTo(n,e-.5),i.quadraticCurveTo(.5,e-.5,.5,e-n),i.lineTo(.5,n),i.quadraticCurveTo(.5,.5,n,.5),i.closePath()}function zh(i={}){let t=i.width??320,e=i.height??160,n=i.left??20,s=i.top??20,r=i.saturation??.47,o=i.lightness??.7,a=Math.min(window.devicePixelRatio||1,2),c=document.createElement("canvas");c.id="deco-hist-overlay",c.width=Math.round(t*a),c.height=Math.round(e*a),c.style.cssText=["position:fixed",`left:${n}px`,`top:${s}px`,`width:${t}px`,`height:${e}px`,"z-index:25","pointer-events:none","display:none"].join(";"),document.body.appendChild(c);let l=c.getContext("2d");l.scale(a,a);let h=!1,u=C=>{h=C,c.style.display=C?"":"none"},p=12,f=22,g=14,y=p,m=f,d=t-p*2,x=e-f-g,_='10px "SF Mono", ui-monospace, monospace';function v(C,w,A){if(!h)return;let P=C.length;if(l.clearRect(0,0,t,e),l.fillStyle="rgba(10,16,18,0.55)",l.strokeStyle="rgba(210,232,220,0.16)",l.lineWidth=1,ho(l,t,e,10),l.fill(),l.stroke(),l.font=_,l.textBaseline="alphabetic",l.fillStyle="#cfe3d6",l.textAlign="left",l.fillText("DECO hue distribution",p,14),l.fillStyle="rgba(207,227,214,0.55)",l.textAlign="right",l.fillText(`t=${A}`,t-p,14),l.strokeStyle="rgba(207,227,214,0.25)",l.beginPath(),l.moveTo(y,m+x+.5),l.lineTo(y+d,m+x+.5),l.stroke(),l.font='9px "SF Mono", ui-monospace, monospace',!(w>0)){l.fillStyle="rgba(207,227,214,0.5)",l.textAlign="center",l.fillText("no DECO alive",t/2,m+x/2),l.textAlign="left",l.fillText("0.63",y,e-3),l.textAlign="right",l.fillText("0.87",y+d,e-3);return}let E=d/P,M=Math.max(1,E-2);for(let R=0;R<P;R++){let F=C[R];if(F<=0)continue;let V=Math.max(1,Math.round(F/w*(x-2))),[z,k,I]=Pc(Bh(R,P),r,o);l.fillStyle=`rgb(${z},${k},${I})`,l.fillRect(y+R*E+(E-M)/2,m+x-V,M,V)}l.fillStyle="rgba(207,227,214,0.55)",l.textAlign="left",l.fillText(String(w),y+2,m+9),l.fillText("0.63",y,e-3),l.textAlign="right",l.fillText("0.87",y+d,e-3)}return{canvas:c,update:v,setVisible:u,get visible(){return h}}}var uo=720,fo=320,p_=uo,pn=10;function On(i,t=2){return Math.round(i)*t}function m_({frameLen:i,historyFrames:t,height:e,newestTick:n,every:s=120}){let r=[];if(!(i>0)||!(e>0))return r;let o=t/e;for(let a=s;a<i;a+=s){let c=Math.round(a*o);c<20||c>e-16||r.push({back:a,y:c,tick:n-On(a,pn)})}return r}function kh(i={}){let t=i.width??fo,e=i.height??p_,n=i.left??20,s=i.top??190,r=i.saturation??.47,o=i.lightness??.7,a=i.historyFrames??uo,c=24,l=12,h=46,u=Math.min(window.devicePixelRatio||1,2),p=document.createElement("canvas");p.id="deco-spectrogram-overlay",p.width=Math.round((t+h)*u),p.height=Math.round(e*u),p.style.cssText=["position:fixed",`left:${n}px`,`top:${s}px`,`width:${t+h}px`,`height:${e}px`,"z-index:25","pointer-events:none","display:none"].join(";"),document.body.appendChild(p);let f=p.getContext("2d");f.scale(u,u);let g=!1,y=z=>{g=z,p.style.display=z?"":"none"},m=new Uint32Array(a*c),d=0,x=0;function _(z){let k=d*c;for(let I=0;I<c;I++)m[k+I]=z[I];d=(d+1)%a,x<a&&(x+=1)}function v(z){if(x===0)return null;let k=Math.abs(z);if(k>=x)return null;let B=((((d-1)%a+a)%a-k)%a+a)%a*c;return m.slice(B,B+c)}function C(){return x===0?0:-(x-1)}let w=new Array(c);for(let z=0;z<c;z++){let k=Bh(z,c);w[z]=Pc(k,r,o)}let A=Pc(.75,r,.18);function P(z,k,I){let Y=f.createImageData(t,e),B=Y.data;for(let Q=0;Q<e;Q++){let ut=Math.round(Q*k),zt=((z-ut)%a+a)%a*c;for(let $t=0;$t<t;$t++){let H=Math.min(c-1,Math.floor($t*c/t)),st=m[zt+H],pt=st>0?Math.min(1,st*I):0,ct=Q*t+$t;B[ct*4]=Math.round(A[0]*(1-pt)+w[H][0]*pt),B[ct*4+1]=Math.round(A[1]*(1-pt)+w[H][1]*pt),B[ct*4+2]=Math.round(A[2]*(1-pt)+w[H][2]*pt),B[ct*4+3]=255}}return Y}function E(z,k=!0){let I=t,Y=e;f.font='10px "SF Mono", ui-monospace, monospace',f.textBaseline="alphabetic",f.textAlign="left",f.fillStyle="#cfe3d6",f.fillText("DECO hue evolution",l,14),f.textAlign="right",f.fillStyle="rgba(207,227,214,0.55)",f.fillText(z,I-l,14),k&&(f.font='9px "SF Mono", ui-monospace, monospace',f.textAlign="left",f.fillStyle="rgba(207,227,214,0.45)",f.fillText("0.63",l,Y-3),f.textAlign="right",f.fillText("0.87",I-l,Y-3),f.textAlign="left",f.fillText("\u2191 newest",l,Y-14),f.textAlign="right",f.fillText("oldest \u2193",I-l,Y-14))}function M(z){let k=m_({frameLen:x,historyFrames:a,height:e,newestTick:z});if(k.length!==0){f.font='10px "SF Mono", ui-monospace, monospace',f.textAlign="left",f.textBaseline="middle",f.lineWidth=1;for(let{y:I,tick:Y}of k)f.strokeStyle="rgba(255,255,255,0.35)",f.beginPath(),f.moveTo(t+.5,I+.5),f.lineTo(t+8.5,I+.5),f.stroke(),f.fillStyle="rgba(255,255,255,0.75)",f.fillText(String(Y),t+11,I+.5);f.textBaseline="alphabetic"}}function R(){if(m.fill(0),d=0,x=0,!g)return;let z=t+h;f.clearRect(0,0,z,e),f.fillStyle="rgba(10,16,18,0.55)",f.strokeStyle="rgba(210,232,220,0.16)",f.lineWidth=1,ho(f,z,e,10),f.fill(),f.stroke(),E("t=\u2014",!1),f.font='10px "SF Mono", ui-monospace, monospace',f.fillStyle="rgba(207,227,214,0.5)",f.textAlign="center",f.fillText("cleared \xB7 waiting for live frames",z/2,e/2),f.textAlign="left"}function F(z,k){if(!g||z<=0)return;let I=t+h,Y=e;f.clearRect(0,0,I,Y),f.fillStyle="rgba(10,16,18,0.55)",f.strokeStyle="rgba(210,232,220,0.16)",f.lineWidth=1,ho(f,I,Y,10),f.fill(),f.stroke();let B=a/Y,Q=1/z;f.putImageData(P(d-1,B,Q),0,0),E(`t=${k}`),M(k)}function V(z,k,I){if(!g||x===0)return;let Y=t+h,B=e;f.clearRect(0,0,Y,B),f.fillStyle="rgba(10,16,18,0.55)",f.strokeStyle="rgba(210,232,220,0.16)",f.lineWidth=1,ho(f,Y,B,10),f.fill(),f.stroke();let Q=a/B,ut=1/k;f.putImageData(P(d-1,Q,ut),0,0);let Et=Math.abs(z),zt=Math.min(B-1,Math.round(Et*Q));f.fillStyle="rgba(255,96,96,0.9)",f.fillRect(l,zt,t-l*2,2),f.fillStyle="rgba(255,255,255,1.0)",f.fillRect(l,zt-1,10,4),E(Et>0?`t=${I} \xB7 frame -${Et}`:`t=${I}`),M(I)}return{canvas:p,pushFrame:_,clear:R,getFrameAt:v,getEarliestReadable:C,render:F,renderScrubbed:V,setVisible:y,get visible(){return g},get writeIdx(){return d},get frameLen(){return x},get bufferSize(){return a}}}function Gh(i,t,e,n,s,r=32){let o=[];if(typeof i.sim_clade_count=="function"){let c=new TextDecoder,l=i.sim_clade_count(t)|0;for(let h=0;h<l&&h<64&&i.sim_get_clade(t,h,e)===1;h++){let u=new DataView(i.memory.buffer,e,40),p=u.getUint32(0,!0),f=u.getUint32(4,!0),g=u.getUint32(12,!0),y=u.getUint32(16,!0),m=u.getUint32(20,!0),d=u.getFloat64(24,!0),x=u.getUint32(32,!0),_=u.getFloat32(36,!0),v=i.sim_clade_name(t,h,s,r)|0,C=c.decode(new Uint8Array(i.memory.buffer,s,Math.max(0,v)));o.push({id:p,parentId:f===4294967295?null:f,kindHash:u.getUint32(8,!0),birthTick:g,deathTick:y===4294967295?null:y,aliveCount:m,centroidEff:d,nicheOffset:_,color:`#${(x&16777215).toString(16).padStart(6,"0")}`,name:C,extinct:y!==4294967295})}}let a=[];if(typeof i.sim_event_count=="function"){let c=i.sim_event_count(t)|0;for(let l=0;l<c&&l<64&&(i.sim_get_event(t,l,n)|0)!==0;l++){let u=new DataView(i.memory.buffer,n,56),p=u.getUint32(0,!0),f=u.getUint32(4,!0),g=u.getUint32(12,!0),y=Math.min(u.getUint32(8,!0),8),m=[];for(let d=0;d<y;d++)m.push(u.getUint32(24+d*4,!0));a.push({tick:p,parentCladeId:f,childCladeIds:m,geneticDistance:u.getFloat64(16,!0),kind:g})}}return{clades:o,events:a}}function Wh(i,t,e,n={}){let s=new Map(i.map(_=>[_.id,_])),r=new Map;for(let _ of i)_.parentId!==null&&_.parentId!==void 0&&s.has(_.parentId)&&(r.has(_.parentId)||r.set(_.parentId,[]),r.get(_.parentId).push(_.id));for(let _ of r.values())_.sort((v,C)=>s.get(v).birthTick-s.get(C).birthTick||v-C);let o=i.filter(_=>_.parentId===null||_.parentId===void 0||!s.has(_.parentId)),a=i.length?Math.min(...i.map(_=>_.birthTick)):0,c=Math.max(1,e-a),l=new Map,h=new Map,u=i.filter(_=>!r.has(_.id)).sort((_,v)=>_.birthTick-v.birthTick||_.id-v.id),p=Math.max(1,u.length);u.forEach((_,v)=>l.set(_.id,p===1?.5:v/(p-1)));let f=(_,v)=>{h.set(_,Math.max(v,h.get(_)??0));let C=r.get(_);if(!C||!C.length)return l.get(_)??.5;let w=0;for(let P of C)w+=f(P,v+1);let A=w/C.length;return l.set(_,A),A};for(let _ of o)f(_.id,0);let g=_=>(_-a)/c,y=new Map;for(let _ of t)for(let v of _.childCladeIds)y.set(v,_.geneticDistance);let m=i.map(_=>({id:_.id,name:_.name,parentId:_.parentId??null,birthTick:_.birthTick,deathTick:_.deathTick??null,extinct:!!_.extinct,aliveCount:_.aliveCount??0,centroidEff:_.centroidEff??0,nicheOffset:_.nicheOffset??0,color:_.color||"#cfe3d6",x:g(_.birthTick),y:l.get(_.id)??.5,tipX:_.extinct?g(_.deathTick??_.birthTick):1,depth:h.get(_.id)??0,geneticDistance:y.get(_.id)??null})),d=[];for(let _ of i)_.parentId===null||_.parentId===void 0||!s.has(_.parentId)||d.push({from:_.parentId,to:_.id,geneticDistance:y.get(_.id)??null,extinct:!!_.extinct,birthTick:_.birthTick});let x=t.map(_=>{let v=s.get(_.parentCladeId);return{tick:_.tick,parentCladeId:_.parentCladeId,geneticDistance:_.geneticDistance,kind:_.kind??0,x:g(_.tick),y:v?l.get(v.id)??.5:.5}});return{version:1,rootTick:a,currentTick:e,fossilHighlight:!!n.fossilHighlight,nodes:m,edges:d,events:x}}function Hh(i,t,e,n){i.beginPath(),i.moveTo(n,.5),i.lineTo(t-n,.5),i.quadraticCurveTo(t-.5,.5,t-.5,n),i.lineTo(t-.5,e-n),i.quadraticCurveTo(t-.5,e-.5,t-n,e-.5),i.lineTo(n,e-.5),i.quadraticCurveTo(.5,e-.5,.5,e-n),i.lineTo(.5,n),i.quadraticCurveTo(.5,.5,n,.5),i.closePath()}var Ic='10px "SF Mono", ui-monospace, monospace',Vh="rgba(160,175,168,0.45)",Ks="#9fb3a8";function Xh(i={}){let t=i.width??360,e=i.height??280,n=i.right??20,s=i.bottom??120,r={top:26,bottom:20,left:84,right:30},o=Math.min(window.devicePixelRatio||1,2),a=document.createElement("canvas");a.id="species-phylogeny-overlay",a.width=Math.round(t*o),a.height=Math.round(e*o),a.style.cssText=["position:fixed",`right:${n}px`,`bottom:${s}px`,`width:${t}px`,`height:${e}px`,"z-index:10","pointer-events:auto","display:none"].join(";"),document.body.appendChild(a);let c=a.getContext("2d");c.scale(o,o);let l=!1,h=null,u=null,p=!1,f=x=>{l=x,a.style.display=x?"":"none"},g=x=>r.left+x*(t-r.left-r.right),y=x=>r.top+x*(e-r.top-r.bottom);function m(x){if(!u||!x)return;let _=null,v=144;for(let M of x.nodes)for(let[R,F]of[[g(M.tipX),y(M.y)],[g(M.x),y(M.y)]]){let V=(R-u.mx)**2+(F-u.my)**2;V<v&&(v=V,_=M)}if(!_)return;let C=[`${_.extinct?"\u2020":""}${_.name}`,`born t=${_.birthTick}`,_.extinct?`extinct t=${_.deathTick}`:`alive \xD7${_.aliveCount}`,_.geneticDistance!=null?`d=${_.geneticDistance.toFixed(2)} vs parent`:null,`eff=${_.centroidEff.toFixed(2)}`,`niche=${(_.nicheOffset??0).toFixed(2)}`].filter(Boolean);c.font=Ic;let w=Math.max(...C.map(M=>c.measureText(M).width))+14,A=C.length*13+8,P=u.mx+12,E=u.my-A-6;P+w>t-4&&(P=u.mx-w-12),E<4&&(E=u.my+12),c.fillStyle="rgba(8,14,16,0.92)",c.strokeStyle="rgba(210,232,220,0.3)",c.lineWidth=1,Hh(c,0,0,0),c.beginPath(),c.rect(P,E,w,A),c.fill(),c.stroke(),c.fillStyle="#e9f2ec",c.textAlign="left",c.textBaseline="alphabetic",C.forEach((M,R)=>{c.fillStyle=R===0?_.extinct?Ks:_.color:"rgba(207,227,214,0.8)",c.fillText(M,P+7,E+15+R*13)})}function d(){if(!l)return;let x=h;if(c.clearRect(0,0,t,e),c.fillStyle="rgba(10,16,18,0.65)",c.strokeStyle="rgba(210,232,220,0.16)",c.lineWidth=1,Hh(c,t,e,10),c.fill(),c.stroke(),c.font='bold 11px "SF Mono", ui-monospace, monospace',c.fillStyle="#e9f2ec",c.textAlign="left",c.textBaseline="alphabetic",c.fillText("SPECIES PHYLOGENY",r.left-60,16),c.font=Ic,c.fillStyle="rgba(207,227,214,0.55)",c.textAlign="right",c.fillText(x?`t=${x.currentTick}`:"t=\u2014",t-10,16),!x||x.nodes.length===0){c.fillStyle="rgba(207,227,214,0.5)",c.textAlign="center",c.fillText("no clades yet",t/2,e/2);return}let _=new Map(x.nodes.map(v=>[v.id,v]));for(let v of x.edges){let C=_.get(v.from),w=_.get(v.to);if(!C||!w)continue;let A=g(C.x),P=y(C.y),E=y(w.y),M=g(w.tipX);c.strokeStyle=v.extinct?x.fossilHighlight?Ks:Vh:w.color,c.lineWidth=1,c.setLineDash(v.extinct?[3,3]:[]),c.beginPath(),c.moveTo(A+.5,P),c.lineTo(A+.5,E+.5),c.lineTo(M,E+.5),c.stroke(),c.setLineDash([]),v.extinct&&x.fossilHighlight&&(c.strokeStyle=Ks,c.strokeRect(M-2.5,E-2.5,5,5))}for(let v of x.events)c.fillStyle=v.kind===2?"#ffb347":v.kind===1?"#8a9a90":"#ff5a5a",c.beginPath(),c.arc(g(v.x),y(v.y),2.5,0,Math.PI*2),c.fill();c.font=Ic,c.textAlign="right",c.textBaseline="middle";for(let v of x.nodes){let C=!x.edges.some(P=>P.from===v.id),w=g(v.tipX),A=y(v.y);c.fillStyle=v.extinct?x.fossilHighlight?Ks:Vh:v.color,c.beginPath(),c.arc(w,A,2.5,0,Math.PI*2),c.fill(),C&&(c.fillStyle=v.extinct?x.fossilHighlight?Ks:"rgba(160,175,168,0.7)":"#e9f2ec",c.fillText(`${v.extinct?"\u2020":""}${v.name}`,r.left-8,A))}c.fillStyle="rgba(207,227,214,0.45)",c.textBaseline="alphabetic",c.textAlign="left",c.fillText(`t=${x.rootTick}`,r.left,e-6),c.textAlign="right",c.fillText(`t=${x.currentTick}`,t-r.right,e-6),m(x)}return a.addEventListener("mousemove",x=>{let _=a.getBoundingClientRect();u={mx:x.clientX-_.left,my:x.clientY-_.top},d()}),a.addEventListener("mouseleave",()=>{u=null,d()}),{canvas:a,update(x){h=x,p=!!x?.fossilHighlight,d()},setVisible:f,render:d,get visible(){return l},get graph(){return h}}}function qh(i,t,e={}){let n=e.overlapThreshold??.15,s=l=>{let h=l.nicheOffset??0;return Number.isFinite(h)?Math.max(-1,Math.min(1,h)):0},r=i.filter(l=>!l.extinct).sort((l,h)=>l.birthTick-h.birthTick||l.id-h.id),o=new Map(r.map((l,h)=>[l.id,r.length===1?.5:h/(r.length-1)])),a=i.map(l=>{let h=s(l);return{id:l.id,name:l.name,niche:h,members:l.aliveCount??0,extinct:!!l.extinct,color:l.color||"#cfe3d6",x:(h- -1)/2,y:l.extinct?.5:o.get(l.id)??.5}}),c=[];for(let l=0;l<r.length;l++)for(let h=l+1;h<r.length;h++){let u=Math.abs(s(r[l])-s(r[h]));u<n&&c.push({a:r[l].id,b:r[h].id,dist:u})}return{version:1,currentTick:t,threshold:n,points:a,overlaps:c}}function g_(i,t,e,n){i.beginPath(),i.moveTo(n,.5),i.lineTo(t-n,.5),i.quadraticCurveTo(t-.5,.5,t-.5,n),i.lineTo(t-.5,e-n),i.quadraticCurveTo(t-.5,e-.5,t-n,e-.5),i.lineTo(n,e-.5),i.quadraticCurveTo(.5,e-.5,.5,e-n),i.lineTo(.5,n),i.quadraticCurveTo(.5,.5,n,.5),i.closePath()}var Yh='10px "SF Mono", ui-monospace, monospace';function $h(i={}){let t=i.width??340,e=i.height??210,n=i.right??400,s=i.bottom??120,r={top:26,bottom:40,left:22,right:22},o=Math.min(window.devicePixelRatio||1,2),a=document.createElement("canvas");a.id="species-niche-overlay",a.width=Math.round(t*o),a.height=Math.round(e*o),a.style.cssText=["position:fixed",`right:${n}px`,`bottom:${s}px`,`width:${t}px`,`height:${e}px`,"z-index:10","pointer-events:auto","display:none"].join(";"),document.body.appendChild(a);let c=a.getContext("2d");c.scale(o,o);let l=!1,h=null,u=g=>r.left+g*(t-r.left-r.right),p=g=>r.top+g*(e-r.top-r.bottom);function f(){if(!l)return;if(c.clearRect(0,0,t,e),c.fillStyle="rgba(10,16,18,0.65)",c.strokeStyle="rgba(210,232,220,0.16)",c.lineWidth=1,g_(c,t,e,10),c.fill(),c.stroke(),c.font='bold 11px "SF Mono", ui-monospace, monospace',c.fillStyle="#e9f2ec",c.textAlign="left",c.textBaseline="alphabetic",c.fillText("NICHE SPACE",r.left,16),c.font=Yh,c.fillStyle="rgba(207,227,214,0.55)",c.textAlign="right",c.fillText(h?`t=${h.currentTick}`:"t=\u2014",t-r.right,16),!h||h.points.length===0){c.fillStyle="rgba(207,227,214,0.5)",c.textAlign="center",c.fillText("no clades yet",t/2,e/2);return}let g=new Map(h.points.map(m=>[m.id,m]));c.strokeStyle="rgba(255,179,71,0.7)",c.lineWidth=1.5;for(let m of h.overlaps){let d=g.get(m.a),x=g.get(m.b);!d||!x||d.extinct||x.extinct||(c.setLineDash([4,3]),c.beginPath(),c.moveTo(u(d.x),p(d.y)),c.lineTo(u(x.x),p(x.y)),c.stroke(),c.setLineDash([]))}let y=p(.5);c.strokeStyle="rgba(207,227,214,0.35)",c.lineWidth=1,c.beginPath(),c.moveTo(u(0),y),c.lineTo(u(1),y),c.stroke(),c.fillStyle="rgba(207,227,214,0.5)",c.font=Yh,c.textAlign="center",c.textBaseline="top";for(let[m,d]of[[0,"-1"],[.5,"0"],[1,"+1"]])c.beginPath(),c.moveTo(u(m),y-3),c.lineTo(u(m),y+3),c.stroke(),c.fillText(d,u(m),e-r.bottom+8);for(let m of h.points){let d=2.5+Math.sqrt(Math.max(0,m.members))*.9;m.extinct?(c.strokeStyle="rgba(160,175,168,0.55)",c.setLineDash([2,2]),c.lineWidth=1,c.beginPath(),c.arc(u(m.x),p(m.y),d+1.5,0,Math.PI*2),c.stroke(),c.setLineDash([]),c.fillStyle="rgba(160,175,168,0.4)"):c.fillStyle=m.color,c.beginPath(),c.arc(u(m.x),p(m.y),d,0,Math.PI*2),c.fill(),h.points.length<=8&&(c.fillStyle=m.extinct?"rgba(160,175,168,0.8)":"rgba(233,242,236,0.9)",c.textAlign="center",c.textBaseline="bottom",c.fillText(`${m.extinct?"\u2020":""}${m.name}`,u(m.x),p(m.y)-d-3))}}return{canvas:a,update(g){h=g,f()},setVisible(g){l=g,a.style.display=g?"":"none",g&&f()},render:f,get visible(){return l},get cloud(){return h}}}function Lc(i,t,e,n,s=5){if(t===0){i.strokeStyle="rgba(255,96,64,0.85)",i.lineWidth=1.2;for(let r=1;r<=3;r++)i.beginPath(),i.moveTo(e-r*s*.8,n-r*s*.8),i.lineTo(e-r*s*.8+s*.5,n-r*s*.8+s*.5),i.stroke();i.fillStyle="#ff5a3c",i.beginPath(),i.arc(e,n,s*.75,0,Math.PI*2),i.fill()}else i.strokeStyle="#ffb347",i.lineWidth=1.6,i.beginPath(),i.moveTo(e,n-s),i.lineTo(e,n+s*.4),i.stroke(),i.fillStyle="#ffb347",i.beginPath(),i.arc(e,n+s*.75,s*.6,0,Math.PI*2),i.fill()}function Zh(i,t,e,n,s){i.strokeStyle=t===0?"rgba(255,96,64,0.55)":"rgba(255,179,71,0.55)",i.lineWidth=1,i.setLineDash([3,3]),i.beginPath(),i.moveTo(e+.5,n),i.lineTo(e+.5,s),i.stroke(),i.setLineDash([])}function Kh(i,t,e,n,s){i.strokeStyle=t===0?"rgba(255,96,64,0.55)":"rgba(255,179,71,0.55)",i.lineWidth=1,i.setLineDash([3,3]),i.beginPath(),i.moveTo(n,e+.5),i.lineTo(s,e+.5),i.stroke(),i.setLineDash([])}function Jh(i,t){let e=i.filter(s=>s.active),n="";if(e.length>0){let s=e[0],r=s.kind===0?"\u2604":"\u{1F321}";n+=`<span style="color:${s.kind===0?"#ff8f6b":"#ffb347"}">${r} t=${s.tick} dust=${(t??1).toFixed(2)} \xB7 \u8FD8\u5269 ${s.remaining}t</span><br>`}if(i.length>0){let s=i.slice(-5).map(r=>`${r.kind===0?"\u2604":"\u{1F321}"}#${r.id}@t=${r.tick}${r.active?"":"\u2020"}`).join(" \xB7 ");n+=`<span style="opacity:0.72">\u707E\u53D8: ${s}</span><br>`}return n}function jh(i={}){let t=document.createElement("div");return t.id="cataclysm-dust-overlay",t.style.cssText=["position:fixed","inset:0","z-index:5","pointer-events:none","background:radial-gradient(circle at 50% 40%, rgba(146,138,124,0.9), rgba(84,80,72,0.95))","opacity:0","display:none","transition:opacity 0.4s"].join(";"),document.body.appendChild(t),{el:t,setDust(e){let n=Math.max(0,Math.min(1,1-e));if(n<.01){t.style.display="none";return}t.style.display="",t.style.opacity=(n*.5).toFixed(3)}}}var __=58,x_=320,Qh="rgba(72,181,255,0.92)",y_="rgba(30,40,60,0.78)",v_="rgba(120,180,220,0.25)";var tu=4,eu=14,nu=16,iu=12;function M_({frameLen:i,trackLeft:t,trackW:e,tickAnchor:n,every:s=60}){let r=[];if(i<2)return r;for(let o=s;o<i-1;o+=s){let a=i-1-o,c=n-On(a,pn);r.push({f:o,x:t+o/(i-1)*e,tick:c,label:o/s%2===0?`t=${c}`:null})}return r}function Dc(i,t,e,n,s,r){i.beginPath(),i.moveTo(t+r,e),i.lineTo(t+n-r,e),i.quadraticCurveTo(t+n,e,t+n,e+r),i.lineTo(t+n,s-r),i.quadraticCurveTo(t+n,s,t+n-r,s),i.lineTo(t+r,s),i.quadraticCurveTo(t,s,t,s-r),i.lineTo(t,e+r),i.quadraticCurveTo(t,e,t+r,e),i.closePath()}function su(i){let t=i.historyBuf;if(!t||typeof t.frameLen!="number")throw new TypeError("createEvolutionScrubber requires a valid spectrogram history buffer");let e=Math.max(x_,i.width??320),n=i.height??__,s=Math.min(window.devicePixelRatio||1,2),r=document.createElement("canvas");r.id="deco-scrubber-overlay",r.style.cssText=["position:fixed","left:20px","top:910px",`width:${e}px`,`height:${n}px`,"z-index:25","pointer-events:auto","display:none"].join(";"),document.body.appendChild(r);let o=r.getContext("2d");o.scale(s,s);let a=!1,c=!0,l=0,h=!0,u=!1,p=0,f=null,g=null,y=k=>{g=k},m=k=>{a=k,r.style.display=k?"":"none",h=!0,k?z():V()},d=k=>{let I=t.getEarliestReadable();return Math.max(I,Math.min(0,k))};function x(k){let I=r.getBoundingClientRect(),Y=Math.max(0,Math.min(1,(k-I.left)/e)),B=t.getEarliestReadable();l=d(Math.round(B*(1-Y))),h=!0,g&&g(!c&&l!==0,l)}function _(k){let I=t.frameLen;if(I===0){l=0,h=!0;return}l=-(I-1-k),l=d(l),h=!0,g&&g(!1,l)}function v(){return c}function C(){c||(c=!0,l=0,h=!0,g&&g(!0,0))}function w(){c&&(c=!1,h=!0,g&&g(!1,l))}let A=k=>{k!==p&&(p=k,h=!0)};function P(){if(!a||!h)return;h=!1,o.clearRect(0,0,e,n);let k=t.frameLen;if(k===0){o.fillStyle="rgba(207,227,214,0.4)",o.font='10px "SF Mono", ui-monospace, monospace',o.textAlign="center",o.fillText("push frames first",e/2,n/2+3);return}let I=t.getEarliestReadable(),Y=nu,B=nu+iu,Q=iu,ut=eu,Et=e-eu,zt=Et-ut;o.fillStyle=y_,Dc(o,ut,Y,zt,Q,3),o.fill(),o.strokeStyle=v_,o.lineWidth=.8,o.stroke();let $t=k<=1?1:Math.abs(l)/(k-1),H=Math.max($t*zt,3),st=o.createLinearGradient(ut,0,ut+H,0);st.addColorStop(0,"rgba(42,130,210,0.78)"),st.addColorStop(1,Qh),o.fillStyle=st,Dc(o,ut,Y,H,Q,3),o.fill();let pt=ut+H,ct=Y+Q/2;if(o.fillStyle=Qh,o.fillRect(pt-tu/2,Y,tu,Q),o.beginPath(),o.moveTo(pt,Y-5),o.lineTo(pt-4,Y),o.lineTo(pt+4,Y),o.closePath(),o.fill(),o.beginPath(),o.moveTo(pt,B+5),o.lineTo(pt-4,B),o.lineTo(pt+4,B),o.closePath(),o.fill(),o.font='9px "SF Mono", ui-monospace, monospace',!c&&l<0){let At=p-On(-l,pn);o.font='bold 12px "SF Mono", ui-monospace, monospace',o.fillStyle="rgba(72,181,255,0.95)",o.textAlign="center",o.fillText(`t=${At}`,Math.max(30,Math.min(e-30,pt)),12)}else o.fillStyle="rgba(207,227,214,0.55)",o.textAlign="left",o.fillText("start",ut,12),o.textAlign="right",o.fillText("now",Et,12);let Pt=M_({frameLen:k,trackLeft:ut,trackW:zt,tickAnchor:p});o.fillStyle="rgba(255,255,255,0.35)";for(let At of Pt)o.fillRect(Math.round(At.x),B+2,1,2);o.font='8px "SF Mono", ui-monospace, monospace',o.textAlign="center";for(let At of Pt)At.label!==null&&(o.fillStyle="rgba(207,227,214,0.55)",o.fillText(At.label,At.x,At.f/60%2===0?B+14:B+24));if(f!==null&&!u){let At=Math.max(0,Math.min(1,f/e)),Ot=Math.round((k-1)*At),Xt=k-1-Ot,Jt=`frame ${Ot} \xB7 t=${p-On(Xt,pn)}`;o.font='8px "SF Mono", ui-monospace, monospace';let le=o.measureText(Jt).width+12,N=Math.max(2,Math.min(e-le-2,f-le/2));o.fillStyle="rgba(10,16,18,0.85)",Dc(o,N,1,le,14,4),o.fill(),o.strokeStyle="rgba(120,180,220,0.35)",o.lineWidth=.8,o.stroke(),o.fillStyle="rgba(207,227,214,0.9)",o.textAlign="center",o.fillText(Jt,N+le/2,11)}}let E=null,M=null,R=null,F=null;function V(){E&&(window.removeEventListener("mousemove",E),E=null),M&&(window.removeEventListener("mouseup",M),M=null),R&&(r.removeEventListener("mousemove",R),R=null),F&&(r.removeEventListener("mouseleave",F),F=null)}function z(){V(),R=k=>{u||(f=k.clientX-r.getBoundingClientRect().left,h=!0)},F=()=>{f=null,h=!0},r.addEventListener("mousemove",R),r.addEventListener("mouseleave",F),r.addEventListener("mousedown",k=>{k.preventDefault(),u=!0,x(k.clientX),E=I=>{u&&x(I.clientX)},M=()=>{u=!1,l=d(l),h=!0,window.removeEventListener("mousemove",E),window.removeEventListener("mouseup",M),E=null,M=null},window.addEventListener("mousemove",E),window.addEventListener("mouseup",M)}),r.addEventListener("click",k=>{u||x(k.clientX)})}return{canvas:r,updatePointer:x,updatePointerFromFrame:_,isPlaying:v,play:C,pause:w,setTickAnchor:A,setVisible:m,setOnChange:y,get visible(){return a},get offset(){return l},get playing(){return c},draw:P,getDirty(){return h},clearDirty(){h=!1},_bindEvents:z,_cleanupEvents:V}}function S_(i){let t=new Uint8Array(i.buffer,i.byteOffset,i.byteLength),e="";for(let n=0;n<t.length;n+=32768)e+=String.fromCharCode.apply(null,t.subarray(n,n+32768));return btoa(e)}function ru({rows:i,buckets:t,hueMin:e,hueSpan:n,capturedTick:s,capturedAt:r}){if(!Array.isArray(i)||i.length===0)throw new Error("encodeEvolutionHistory: rows \u4E3A\u7A7A");let o=i[0].length;if(o!==t)throw new Error(`encodeEvolutionHistory: \u7B2C\u4E00\u884C\u5BBD\u5EA6 ${o} \u2260 \u671F\u671B\u6876\u6570 ${t}`);let a=new Uint32Array(i.length*o);for(let c=0;c<i.length;c++){if(i[c].length!==o)throw new Error(`encodeEvolutionHistory: \u7B2C ${c} \u884C\u5BBD\u5EA6 ${i[c].length} \u2260 ${o}`);a.set(i[c],c*o)}return JSON.stringify({version:1,capturedAt:r??`tick ${s}`,historyFrames:i.length,buckets:t,hueMin:e,hueSpan:n,data:S_(a)})}var E_=/^tick (\d+)$/;function ou(){let i="live",t=0,e=[],n=null,s=null,r=()=>{s&&s(i)};function o(l){return i!=="live"||(t+=1),t}function a(l){let h=JSON.parse(l);if(!h||h.version!==1)throw new Error(`\u4E0D\u652F\u6301\u7684 evolution history schema version=${h&&h.version}`);let u=h.buckets;if(!Number.isInteger(u)||u<=0)throw new Error(`buckets \u975E\u6CD5: ${u}`);let p=atob(h.data),f=new Uint8Array(p.length);for(let x=0;x<p.length;x++)f[x]=p.charCodeAt(x);let g=new Uint32Array(f.buffer);if(g.length%u!==0)throw new Error(`data \u89E3\u51FA ${g.length} \u4E2A u32\uFF0C\u4E0D\u662F buckets=${u} \u7684\u6574\u6570\u500D`);let y=g.length/u;if(Number.isInteger(h.historyFrames)&&h.historyFrames!==y)throw new Error(`historyFrames=${h.historyFrames} \u2260 data \u884C\u6570 ${y}`);let m=E_.exec(h.capturedAt??""),d=m?Number(m[1]):0;e=[];for(let x=0;x<y;x++)e.push({frameIndex:x,tick:Math.max(0,d-(y-1-x)*pn),counts:g.slice(x*u,(x+1)*u)});return n={capturedAt:h.capturedAt??"",capturedTick:d,frames:y,buckets:u,hueMin:h.hueMin,hueSpan:h.hueSpan},i="loaded",r(),n}function c(){i="live",t=0,e=[],n=null,r()}return{pushLive:o,loadFromJSON:a,clear:c,getState:()=>i,getLiveCount:()=>t,getLoadedRows:()=>e,getLoadedInfo:()=>n,setOnStateChange:l=>{s=l}}}var Nc=1024,mo=64,L_=1444*4,hu=L_,uu=0,du=4,Fc=8;var D_=28,U_=32,N_=36,F_=40;var O_=1;var Js=48,Oc=hu+Nc*Js,Bc=Oc+mo*8,zc=Bc+6*4,B_=5,fu=zc+64*4,pu=fu+64*40,z_=pu+64*56,go=["herb","pred","scav","omn","fish","deco"],kc={h:"herb",p:"pred",s:"scav",o:"omn",f:"fish",d:"deco"},k_=20261102,cu=300,po=2,Hc=28800,H_=1080,Vc={herb:{cap:420,yOff:i=>.9*i,geo:"sphere"},pred:{cap:110,yOff:i=>1*i,geo:"cone"},scav:{cap:40,yOff:i=>.8*i,geo:"coneSmall"},omn:{cap:80,yOff:i=>.8*i,geo:"sphereSmall"},fish:{cap:60,yOff:()=>-.8,geo:"fish"},deco:{cap:80,yOff:i=>.4*i,geo:"deco"}};async function V_(){let i=await fetch("./ecology-sim.wasm");if(!i.ok)throw new Error(`fetch ecology-sim.wasm: ${i.status}`);if(i.headers.get("content-type")?.includes("application/wasm")&&"instantiateStreaming"in WebAssembly)return(await WebAssembly.instantiateStreaming(i,{})).instance;let t=new Uint8Array(await i.arrayBuffer());return(await WebAssembly.instantiate(t,{})).instance}function G_(i){let t=i.exports,e=t.sim_new(k_),n=t.sim_scratch(e),s=null,r=0;function o(){let a=t.memory.buffer;return s&&a.byteLength===r||(r=a.byteLength,s={daylight:new Float64Array(a,n+Oc,mo),counts:new Uint32Array(a,n+Bc,6),hist:new Uint32Array(a,n+zc,24),mineral:new Float32Array(a,n,1444),agents:new DataView(a,n+hu,Nc*Js)}),s}return{exports:t,views:o,step(a,c){t.sim_step(e,a,n+Oc,c)},readAgents(){return t.sim_agents_flat(e,Nc)},readMineral(){t.sim_mineral_resampled(e)},readCounts(){t.sim_counts(e)},tick(){return t.sim_tick(e)},season(){return t.sim_season(e)},mineralPeak(){return t.sim_mineral_peak(e)},decoEffMean(){return t.sim_deco_eff_mean(e)},decoEffStd(){return t.sim_deco_eff_std(e)},readDecoHistogram(a=24){return t.sim_deco_hue_histogram(e,a,n+zc)},readPhylogeny(){return Gh(t,e,n+fu,n+pu,n+z_,32)},triggerCataclysm(a,c,l,h,u,p){return t.sim_trigger_cataclysm(e,a,c,l,h,u,p)|0},cataclysmCount(){return t.sim_cataclysm_count(e)|0},getCataclysm(a,c){return t.sim_get_cataclysm(e,a,c)|0},envMods(a){return t.sim_env_mods(e,a)|0},getBufferLen(){return t.memory.buffer.byteLength},rawCountsBytes(){let a=new Uint8Array(t.memory.buffer,n+Bc,24);return Array.from(a).map(c=>c.toString(16).padStart(2,"0")).join(" ")},dispose(){t.sim_free(e)}}}function W_(i){switch(i){case"herb":return new $e(1,10,8);case"pred":{let t=new Cn(.9,2.6,8);return t.rotateX(Math.PI/2),t}case"scav":{let t=new Cn(.7,2,7);return t.rotateX(Math.PI/2),t}case"omn":return new $e(.85,9,7);case"fish":{let t=new Cn(.45,2.2,6);return t.rotateX(Math.PI/2),t}case"deco":return new Zr(.7,0);default:return new $e(1,8,6)}}function X_(i){let t={};for(let e of go){let n=Vc[e],s=W_(e),r=new Kr({color:16777215}),o=new An(s,r,n.cap);o.instanceMatrix.setUsage(so),o.instanceColor=new gi(new Float32Array(n.cap*3),3),o.instanceColor.setUsage(so),o.frustumCulled=!1,o.count=0,o.castShadow=e!=="fish",i.add(o),t[e]=o}return t}var _o=new ce,xo=new U,mu=new qe,yo=new U;function Y_(i,t,e){let s=t.views().agents,r={herb:0,pred:0,scav:0,omn:0,fish:0,deco:0};for(let o of go)i[o].count=0;for(let o=0;o<e;o++){let a=o*Js,c=s.getUint8(a+Fc),l=kc[c];if(!l)continue;let h=i[l];if(h.count>=Vc[l].cap)continue;let u=s.getFloat32(a+uu,!0),p=s.getFloat32(a+du,!0),f=s.getFloat32(a+U_,!0),g=s.getUint32(a+D_,!0),y=s.getUint32(a+N_,!0),m=Vc[l],d=l==="fish"?0-.8+Math.sin(o*2.1)*.3:_e(u,p)+m.yOff(f);xo.set(u,d,p),yo.setScalar(f),_o.compose(xo,mu,yo),h.setMatrixAt(h.count,_o);let x=h.count*3,_=h.instanceColor.array;_[x]=(g>>>16&255)/255,_[x+1]=(g>>>8&255)/255,_[x+2]=(g&255)/255,y&O_&&(_[x]+=(1-_[x])*.22,_[x+1]+=(1-_[x+1])*.22,_[x+2]+=(1-_[x+2])*.22),h.count+=1,r[l]+=1}for(let o of go)i[o].instanceMatrix.needsUpdate=!0,i[o].instanceColor.needsUpdate=!0;return r}function lu(i){let t=i*.1*24/Hc%24,e=Math.sin((t-6)/12*Math.PI);return _i((e+.03)/.27,0,1)}function q_(){let i=document.createElement("div");return i.id="wasm-hud",i.style.cssText=["position:fixed","top:max(18px, env(safe-area-inset-top))","right:max(20px, env(safe-area-inset-right))","z-index:30","pointer-events:none","text-align:right",'font:11px/1.7 "SF Mono", ui-monospace, monospace',"color:#cfe3d6","text-shadow:0 1px 3px rgba(0,0,0,0.8)","background:rgba(10,16,18,0.5)","padding:10px 14px","border-radius:10px","border:1px solid rgba(210,232,220,0.14)"].join(";"),document.body.appendChild(i),i}function $_(){let i=document.createElement("div");i.id="wasm-toast",i.style.cssText=["position:fixed","left:50%","bottom:96px","transform:translateX(-50%)","z-index:40","pointer-events:none","font:12px/1.5 inherit","color:#e9eee9","background:rgba(10,16,18,0.78)","padding:8px 18px","border-radius:999px","border:1px solid rgba(210,232,220,0.2)","opacity:0","transition:opacity 0.25s","white-space:nowrap"].join(";"),document.body.appendChild(i);let t=0;return e=>{i.textContent=e,i.style.opacity="1",clearTimeout(t),t=setTimeout(()=>{i.style.opacity="0"},2200)}}async function Z_(i="stage"){let t=document.getElementById(i),e=$_(),n=q_(),s=document.getElementById("load-status"),r=et=>{s&&(s.textContent=et)};r("\u52A0\u8F7D WASM \u751F\u6001\u5185\u6838\u2026");let o=await V_(),a=typeof o.exports.sim_trigger_cataclysm=="function",c=0;if(a){let et=o.exports.memory.buffer.byteLength/65536;o.exports.memory.grow(1),c=et*65536}let l=G_(o),h=l.views();r("\u9884\u70ED\u751F\u6001\uFF08300 ticks\uFF09\u2026");for(let et=0;et<cu;et+=mo){let ot=Math.min(mo,cu-et);for(let $=0;$<ot;$++)h.daylight[$]=lu(l.tick()+$);l.step(ot,ot)}let u=l.decoEffMean(),p=l.decoEffStd();r("\u751F\u6210\u5730\u5F62\u4E0E\u5929\u7A7A\u2026");let{renderer:f,scene:g,camera:y,controls:m}=Dh(t),{env:d,landmarks:x}=Uh(g,f),_=X_(g),{mesh:v,texture:C}=Oh(H_,38);v.visible=!1,g.add(v),r("\u5C31\u7EEA");let w=document.getElementById("enter"),A=document.getElementById("loading");if(w){w.disabled=!1;let et=document.getElementById("enter-label");et&&(et.textContent="\u8FDB\u5165\u4E16\u754C"),w.addEventListener("click",()=>{w.disabled||A&&(A.classList.add("hidden"),setTimeout(()=>A.remove(),1400))})}let P=new Float32Array(1444);function E(et){let ot=l.views().agents,$=[0,0,0,0,0,0];for(let nt=0;nt<et;nt++){let gt=kc[ot.getUint8(nt*Js+Fc)];gt&&($[go.indexOf(gt)]+=1)}return $}let M=!1,R=!1,F=0,V=0,z=0,k=0,I=0,Y=typeof o.exports.sim_deco_hue_histogram=="function",B=Y?zh():null,Q=!0,ut=new Uint32Array(24),Et=0,zt=0,$t=0,H=Y?kh():null,st=!0,pt=0,ct=0,Pt=0,At=ou(),Ot=typeof o.exports.sim_clade_count=="function",Xt=Ot?Xh():null,Jt=!0,le=!1,N=[],de=[],Yt=new Set,qt=new Set,Vt=-1/0,Lt=Ot?$h():null,Ut=!0,T=a?jh():null,S=[],X=-1/0;function j(et=!1){if(!a)return;let ot=l.tick();if(!et&&ot-X<60)return;X=ot;let $=Math.min(l.cataclysmCount(),64),nt=[];for(let gt=0;gt<$&&l.getCataclysm(gt,c)!==0;gt++){let xt=new DataView(o.exports.memory.buffer,c,40);nt.push({id:xt.getUint32(0,!0),kind:xt.getUint32(4,!0),tick:xt.getUint32(8,!0),endsAt:xt.getUint32(12,!0),remaining:xt.getUint32(16,!0),x:xt.getFloat32(20,!0),z:xt.getFloat32(24,!0),radius:xt.getFloat32(28,!0),severity:xt.getFloat32(32,!0),active:xt.getUint32(36,!0)===1})}S=nt}function at(){if(!(!a||S.length===0)){if(Xt&&Xt.visible&&Xt.graph){let et=Xt.graph,ot=360,$=280,nt={top:26,bottom:20,left:84,right:30},gt=Xt.canvas.getContext("2d"),xt=Math.max(1,et.currentTick-et.rootTick);for(let te of S){if(te.tick<et.rootTick)continue;let He=nt.left+(te.tick-et.rootTick)/xt*(ot-nt.left-nt.right);Zh(gt,te.kind,He,nt.top,$-nt.bottom),Lc(gt,te.kind,He,nt.top+12)}}if(H&&H.visible&&ct>0){let et=H.canvas.getContext("2d");for(let ot of S){let $=(ct-ot.tick)/pn;if($<0||$>=uo)continue;let nt=Math.round($);Kh(et,ot.kind,nt,12,fo-12),Lc(et,ot.kind,fo-22,Math.max(9,nt-7))}}}}function J(){Xt&&Xt.update(Wh(N,de,l.tick(),{fossilHighlight:le}))}function Ft(){Lt&&Lt.update(qh(N,l.tick()))}function St(et=!1){if(!Xt)return;let ot=l.tick();if(!et&&ot-Vt<60)return;Vt=ot;let $=l.readPhylogeny();for(let nt of $.clades){if(!Yt.has(nt.id)&&(Yt.add(nt.id),!et&&nt.parentId!==null)){let gt=$.events.find(xt=>xt.childCladeIds.includes(nt.id));e(`\u{1F9EC} ${nt.name} born @ t=${nt.birthTick}`+(gt?` \xB7 d=${gt.geneticDistance.toFixed(2)} vs parent`:""))}if(nt.extinct&&!qt.has(nt.id)&&(qt.add(nt.id),!et)){let gt=$.events.find(xt=>xt.parentCladeId===nt.id&&(xt.kind??0)>=1);e(gt&&gt.kind===2?`\u2694 ${nt.name} \u7ADE\u4E89\u6392\u65A5\u706D\u7EDD @ t=${nt.deathTick}`:`\u26B0 ${nt.name} extinct @ t=${nt.deathTick}`)}}N=$.clades,de=$.events,J(),Ft()}let Rt=-1/0,re=0,lt=new to,K=H?su({historyBuf:H}):null;K&&(K.canvas.style.top="918px",K.setVisible(!0),K.draw());function Nt(){if(!B||!Y)return;let et=l.readDecoHistogram(),ot=l.views().hist;ut.set(ot.subarray(0,et)),Et=0,zt=0;for(let $=0;$<et;$++)Et+=ut[$],ut[$]>ut[zt]&&(zt=$);if($t=l.tick(),At.getState()==="loaded"){if(At.pushLive(ut),!H||!st)return;let $=K&&!K.isPlaying(),nt=$?Math.abs(K.offset):0,gt=H.getFrameAt($?K.offset:0);if(!gt)return;let xt=0;for(let He=0;He<gt.length;He++)gt[He]>xt&&(xt=gt[He]);let te=ct-On(nt,pn);B.update(gt,xt,te),$?H.renderScrubbed(K.offset,xt,te):H.render(pt,ct);return}if(B.update(ut,Et>0?ut[zt]:0,$t),!(!H||!st||Et<=0))if(K&&!K.isPlaying()){let $=H.getFrameAt(K.offset);if($){let nt=0;for(let xt=0;xt<$.length;xt++)$[xt]>nt&&(nt=$[xt]);let gt=ct-On(Math.abs(K.offset),pn);B.update($,nt,gt),H.renderScrubbed(K.offset,nt,gt)}}else H.pushFrame(ut),At.pushLive(ut),pt=Math.max(pt,ut[zt]),ct=$t,Pt+=1,K&&(K.setTickAnchor(ct),K.draw()),H.render(pt,ct)}K&&K.setOnChange((et,ot)=>{H&&st&&(Et>0||At.getState()==="loaded")&&Nt(),K&&K.draw()}),B&&(B.setVisible(!0),H&&H.setVisible(!0),Nt()),Xt&&(Xt.setVisible(!0),St(!0)),Lt&&(Lt.setVisible(!0),Ft()),a&&(j(!0),T.setDust(1)),window.addEventListener("keydown",et=>{if(et.code==="KeyM")M=!M,v.visible=M,e(`\u77FF\u5316\u70ED\u529B\u5C42 ${M?"ON":"OFF"}\uFF08M \u5207\u6362\uFF09`);else if(et.code==="Space")et.preventDefault(),K?(K.isPlaying()?(K.pause(),e("\u751F\u6001\u65F6\u95F4\u5DF2\u6682\u505C")):(K.play(),e("\u751F\u6001\u7EE7\u7EED\u6F14\u5316")),Nt(),K.draw()):(R=!R,e(R?"\u751F\u6001\u65F6\u95F4\u5DF2\u6682\u505C":"\u751F\u6001\u7EE7\u7EED\u6F14\u5316"));else if(et.code==="KeyN"&&B)Q=!Q,B.setVisible(Q),Q&&Nt(),e(`DECO hue \u76F4\u65B9\u56FE ${Q?"ON":"OFF"}\uFF08N \u5207\u6362\uFF09`);else if(et.code==="KeyP"&&H)st=!st,H.setVisible(st),st&&Nt(),e(`DECO hue \u6F14\u5316\u5149\u8C31 ${st?"ON":"OFF"}\uFF08P \u5207\u6362\uFF09`);else if(K&&et.code==="ArrowLeft"){et.preventDefault();let ot=-10,$=H.frameLen;if($>0){K.pause();let nt=Math.max(H.getEarliestReadable(),K.offset+ot),gt=$-1+nt;K.updatePointerFromFrame(Math.max(0,gt)),K.draw(),Nt()}e("\u2190 \u56DE\u9000 10 \u5E27")}else if(K&&et.code==="ArrowRight"){et.preventDefault();let ot=10,$=H.frameLen;if($>0){let nt=Math.min(0,K.offset+ot),gt=$-1+nt;K.updatePointerFromFrame(Math.min($-1,gt)),K.draw(),Nt()}e("\u2192 \u524D\u8FDB 10 \u5E27")}else if(K&&et.code==="Home")et.preventDefault(),H.frameLen>0&&(K.pause(),K.updatePointerFromFrame(0),K.draw(),Nt()),e("\u8DF3\u5230\u6700\u65E9\u5E27");else if(K&&et.code==="End")et.preventDefault(),H.frameLen>0&&(K.play(),K.draw(),Nt()),e("\u8DF3\u5230\u6700\u65B0\u5E27");else if(et.code==="KeyU"&&Lt)Ut=!Ut,Lt.setVisible(Ut),Ut&&Ft(),e(`niche space ${Ut?"ON":"OFF"}\uFF08U \u5207\u6362\uFF09`);else if(et.code==="KeyT"&&Xt)Jt=!Jt,Xt.setVisible(Jt),Jt&&J(),e(`species phylogeny ${Jt?"ON":"OFF"}\uFF08T \u5207\u6362\uFF09`);else if(et.code==="KeyF"&&Xt)le=!le,J(),e(`fossil highlight ${le?"ON":"OFF"}\uFF08F \u5207\u6362\uFF09`);else if(et.code==="KeyC"&&a){let ot=l.triggerCataclysm(0,140,-90,170,.9,600);ot>0?(j(!0),e(`\u2604 \u5C0F\u884C\u661F\u649E\u51FB\u5DF2\u6CE8\u5165 (#${ot}) \xB7 \u5C18\u5E55 600t`)):e("\u6CE8\u5165\u5931\u8D25\uFF08\u53C2\u6570\u65E0\u6548\uFF09")}else if(et.code==="KeyX"&&a){let ot=l.triggerCataclysm(1,0,0,0,.5,1200);ot>0?(j(!0),e(`\u{1F321} \u6C14\u5019\u7A81\u53D8\u5DF2\u6CE8\u5165 (#${ot}) \xB7 \u751F\u957F \xD70.5 \xB7 1200t`)):e("\u6CE8\u5165\u5931\u8D25\uFF08\u53C2\u6570\u65E0\u6548\uFF09")}else if(et.code==="KeyS")try{let ot=document.createElement("canvas");ot.width=t.width,ot.height=t.height;let $=ot.getContext("2d");$.drawImage(t,0,0);for(let yt of[B,H,Xt,Lt])yt&&yt.visible&&$.drawImage(yt.canvas,0,0);let nt=K&&!K.isPlaying(),gt=nt?Math.abs(K.offset):0,xt=ct-On(gt,pn),te=nt&&H?H.getFrameAt(gt):ut,He=0;if(te){let yt=0;for(let Tt=0;Tt<te.length;Tt++)yt+=te[Tt];He=yt/24}let Ze=`tick=${xt} \xB7 frame=${Pt-gt}/${Pt} \xB7 DECO eff=${l.decoEffMean().toFixed(2)} \xB1 ${l.decoEffStd().toFixed(2)}${nt?" (live)":""} \xB7 mean bucket=${He.toFixed(2)}/${24}`,fe=ot.width/Math.max(1,t.clientWidth||ot.width);$.font=`bold ${Math.round(12*fe)}px "SF Mono", ui-monospace, monospace`;let zn=$.measureText(Ze).width,kn=10*fe,Z=10*fe,it=22*fe;$.fillStyle="rgba(10,16,18,0.72)",$.fillRect(kn,Z,zn+16*fe,it),$.fillStyle="#e9f2ec",$.textBaseline="middle",$.fillText(Ze,kn+8*fe,Z+it/2);let ft=ot.width-26*fe,rt=ot.height-26*fe;$.fillStyle="rgba(10,16,18,0.72)",$.beginPath(),$.arc(ft,rt,14*fe,0,Math.PI*2),$.fill(),$.strokeStyle="#e9f2ec",$.lineWidth=Math.max(1.5,2*fe),$.lineCap="round",$.lineJoin="round",$.beginPath(),$.moveTo(ft,rt-6*fe),$.lineTo(ft,rt+2*fe),$.moveTo(ft-4*fe,rt-1*fe),$.lineTo(ft,rt+3*fe),$.lineTo(ft+4*fe,rt-1*fe),$.moveTo(ft-5*fe,rt+6*fe),$.lineTo(ft+5*fe,rt+6*fe),$.stroke();let mt=ot.toDataURL("image/png");localStorage.setItem(`deco-snapshot-${Date.now()}`,mt),ot.toBlob(yt=>{if(!yt)return;let Tt=URL.createObjectURL(yt),kt=document.createElement("a");kt.href=Tt,kt.download=`aether-snapshot-t${xt}.png`,kt.click(),setTimeout(()=>URL.revokeObjectURL(Tt),3e3)},"image/png"),Rt=lt.elapsedTime+3,re=xt,e(`\u5FEB\u7167\u5DF2\u4FDD\u5B58 + \u5DF2\u89E6\u53D1\u4E0B\u8F7D\uFF08t=${xt}\uFF09`)}catch(ot){e(`\u4FDD\u5B58\u5931\u8D25: ${ot.message}`)}else if(et.code==="KeyE"&&H)try{if(H.frameLen===0)e("\u5149\u8C31 buffer \u4E3A\u7A7A\uFF0C\u65E0\u53EF\u5BFC\u51FA\u6570\u636E");else{let ot=[];for(let nt=H.frameLen-1;nt>=0;nt--)ot.push(H.getFrameAt(nt));let $=ru({rows:ot,buckets:24,hueMin:.629,hueSpan:.242,capturedTick:l.tick()});localStorage.setItem("aether-evolution-history",$),e(`\u2713 saved ${H.frameLen}\xD7${24} evolution history`)}}catch(ot){e(`\u5BFC\u51FA\u5931\u8D25: ${ot.message}`)}else if(et.code==="KeyR"&&H)if(At.getState()==="loaded")It(),e("\u5DF2\u6E05\u7A7A\u52A0\u8F7D\u7684\u5386\u53F2\uFF0C\u56DE\u5230 live \u6A21\u5F0F");else{let ot=Zt();e(ot.ok?`Loaded ${ot.loaded}\xD7${ot.buckets} frames from session ${ot.capturedAt}\uFF08\u518D\u6309 R \u6E05\u9664\uFF09`:ot.err)}});function Zt(){let et=localStorage.getItem("aether-evolution-history");if(!et)return{ok:!1,err:"No saved evolution history\uFF08\u5148\u6309 E \u952E\u5BFC\u51FA\u4E00\u4EFD\uFF09"};let ot;try{ot=At.loadFromJSON(et)}catch(xt){return{ok:!1,err:`\u52A0\u8F7D\u5931\u8D25\uFF1A${xt.message}`}}let $=At.getLoadedRows();H.clear();let nt=0;for(let xt of $)for(let te=0;te<xt.counts.length;te++)xt.counts[te]>nt&&(nt=xt.counts[te]);pt=Math.max(pt,nt);for(let xt of $)H.pushFrame(xt.counts);ct=ot.capturedTick,Pt=$.length,K&&(K.play(),K.setTickAnchor(ct),K.draw());let gt=H.getFrameAt(0);if(gt){let xt=0;for(let te=0;te<gt.length;te++)gt[te]>xt&&(xt=gt[te]);B.update(gt,xt,ct)}return H.render(pt,ct),{ok:!0,loaded:$.length,buckets:ot.buckets,capturedAt:ot.capturedAt}}function It(){H.clear(),At.clear(),Pt=0,pt=0,ct=0,K&&(K.play(),K.draw()),Nt()}n.addEventListener("click",et=>{et.target&&et.target.id==="loaded-badge"&&At.getState()==="loaded"&&(It(),e("\u5DF2\u6E05\u7A7A\u52A0\u8F7D\u7684\u5386\u53F2\uFF0C\u56DE\u5230 live \u6A21\u5F0F"))});let ne=96,Kt=.3,Gt=new Tn({color:16777215,transparent:!0,blending:pi,depthWrite:!1}),L=new An(new $e(1,8,6),Gt,ne);L.instanceMatrix.setUsage(so),L.instanceColor=new gi(new Float32Array(ne*3),3),L.frustumCulled=!1,L.count=0,g.add(L);let dt=[],q=new Map;function tt(et){let ot=l.views().agents,$=new Map;for(let nt=0;nt<et;nt++){let gt=nt*Js,xt=kc[ot.getUint8(gt+Fc)];xt&&$.set(ot.getUint32(gt+F_,!0),{x:ot.getFloat32(gt+uu,!0),z:ot.getFloat32(gt+du,!0),kind:xt})}for(let[nt,gt]of q)$.has(nt)||(dt.length>=ne&&dt.shift(),dt.push({x:gt.x,y:gt.kind==="fish"?0-.6:_e(gt.x,gt.z)+.9,z:gt.z,born:lt.elapsedTime}));q=$}function wt(et){for(let $=dt.length-1;$>=0;$--)et-dt[$].born>Kt&&dt.splice($,1);let ot=0;for(let $ of dt){let nt=(et-$.born)/Kt;xo.set($.x,$.y,$.z),yo.setScalar(.8+nt*1.4),_o.compose(xo,mu,yo),L.setMatrixAt(ot,_o);let gt=L.instanceColor.array,xt=(1-nt)*(1-nt);gt[ot*3]=gt[ot*3+1]=gt[ot*3+2]=xt,ot+=1}L.count=ot,L.instanceMatrix.needsUpdate=!0,L.instanceColor.needsUpdate=!0}function bt(){requestAnimationFrame(bt);let et=Math.min(.05,Math.max(0,lt.getDelta()));z+=et,k+=1;let ot=lt.elapsedTime,$=l.tick()*.1*24/Hc%24;if(d.setTime($),Nh(d,m.target),d.update(et,ot),x.update(et,ot),m.update(),!R){for(let gt=0;gt<po;gt++)h.daylight[gt]=lu(l.tick()+gt);l.step(po,po);let nt=l.readAgents();if(Y_(_,l,nt),tt(nt),M){l.readMineral();let gt=0;for(let te=0;te<1444;te++)h.mineral[te]>gt&&(gt=h.mineral[te]);let xt=1/Math.max(gt,.03);for(let te=0;te<1444;te++)P[te]=Math.min(1,h.mineral[te]*xt);Fh(C,P)}Q&&F%B_===0&&Nt(),F+=1,Xt&&St(),a&&(j(),l.envMods(c),T.setDust(new DataView(o.exports.memory.buffer,c,16).getFloat64(0,!0)))}if(wt(lt.elapsedTime),a&&at(),f.render(g,y),ot-V>.25){V=ot,I=k/Math.max(z,1e-6),z=0,k=0;let nt=E(l.readAgents()),gt=Math.floor(l.tick()*.1*24/Hc%24),xt=l.decoEffMean(),te=l.decoEffStd(),He=xt>0?`DECO eff: ${xt.toFixed(2)} \xB1 ${te.toFixed(2)} (${xt>=u?"\u2191":"\u2193"} ${(xt-u).toFixed(2)} / \u03C3${te<=p?"\u2193":"\u2191"} ${Math.abs((te-p)/Math.max(p,1e-9)*100).toFixed(0)}%)<br>`:"",Ze="";if(Xt&&N.length>0){let rt=N.filter(Tt=>!Tt.extinct).length,mt=N.map(Tt=>Tt.extinct?`\u2020${Tt.name}`:Tt.name);Ze=`<span style="color:#c9a0ff">clades: ${mt.length<=5?mt.join(" \xB7 "):`${rt} alive / ${mt.length} total`} \xB7 ${de.length} event${de.length===1?"":"s"}</span><br>`}let fe="";if(a&&S.length>0){l.envMods(c);let rt=new DataView(o.exports.memory.buffer,c,16).getFloat64(0,!0);fe=Jh(S,rt)}let zn="";if(H&&Pt>0){let rt=K&&!K.isPlaying(),mt=rt?Math.abs(K.offset):0,yt=ct-On(mt,pn);zn=`frame: ${Pt-mt} / ${Pt} (tick ${yt})`+(rt?' <span style="color:#ff6b6b;font-weight:bold">[scrub]</span>':"")+"<br>"}let kn=.629+(zt+.5)/24*.242,Z=B&&Q&&Et>0?`<span style="opacity:0.72">hue peak bucket ${zt+1}/${24} \xB7 hue\u2248${kn.toFixed(2)} \xB7 width ${(.242/24).toFixed(4)} \xB7 t=${$t}</span><br>`:"",it="";At.getState()==="loaded"&&(it=`<span id="loaded-badge" style="pointer-events:auto;cursor:pointer;color:#8fd3ff;text-decoration:underline dotted">\u23CE loaded: ${At.getLoadedRows().length}/${H.bufferSize} frames \xB7 \u70B9\u51FB\u6E05\u9664</span><br>`);let ft=ot<Rt?`<span style="color:#8fd3ff">\u2193 PNG ready (t=${re})</span><br>`:"";n.innerHTML=`<b>WASM SNAPSHOT v3</b> \xB7 tick ${l.tick()} \xB7 D+${gt}h<br>\u7267 ${nt[0]} \xB7 \u63A0 ${nt[1]} \xB7 \u8150 ${nt[2]} \xB7 \u6742 ${nt[3]} \xB7 \u9C7C ${nt[4]} \xB7 \u5206\u89E3 ${nt[5]}<br>`+He+Ze+fe+zn+it+Z+`mineral peak ${l.mineralPeak().toFixed(3)} \xB7 M \u70ED\u529B\u5C42${M?" ON":" OFF"}<br>${I.toFixed(0)} fps \xB7 mem ${(l.getBufferLen()/1024).toFixed(0)}KB \xB7 ${R?"PAUSED":`${po} ticks/frame`}`+ft}}return bt(),{handle:l,meshes:_,mineralPlane:v}}typeof document<"u"&&document.getElementById("stage")&&Z_("stage").catch(i=>{console.error("[wasm-viewer] boot failed:",i);let t=document.getElementById("load-status");t&&(t.textContent="\u52A0\u8F7D\u5931\u8D25 \xB7 "+i.message,t.style.color="#e8a07a")});})();
/*! Bundled license information:

three/build/three.module.js:
  (**
   * @license
   * Copyright 2010-2024 Three.js Authors
   * SPDX-License-Identifier: MIT
   *)
*/
