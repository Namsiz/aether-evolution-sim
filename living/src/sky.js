import * as THREE from 'three';
import { Sky } from 'three/addons/objects/Sky.js';
import { Water } from 'three/addons/objects/Water.js';
import { mulberry32, terrainHeight } from './world.js';

const CLOUD_VERT = /* glsl */ `
varying vec3 vDir;
void main() {
  vDir = normalize(position);
  gl_Position = projectionMatrix * modelViewMatrix * vec4(position, 1.0);
}`;

const CLOUD_FRAG = /* glsl */ `
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
}`;

function makeCloudDome() {
  const material = new THREE.ShaderMaterial({
    uniforms: {
      uTime: { value: 0 },
      uCoverage: { value: 0.55 },
      uOpacity: { value: 0.62 },
      uSun: { value: 1 },
      uLit: { value: new THREE.Color(0xffffff) },
      uShadow: { value: new THREE.Color(0x8796aa) },
      uSunDir: { value: new THREE.Vector3(0.3, 0.8, 0.2) },
    },
    vertexShader: CLOUD_VERT,
    fragmentShader: CLOUD_FRAG,
    transparent: true,
    depthWrite: false,
    depthTest: true,
    side: THREE.BackSide,
    fog: false,
  });
  const mesh = new THREE.Mesh(new THREE.SphereGeometry(7200, 32, 20), material);
  mesh.renderOrder = -940;
  mesh.frustumCulled = false;
  return mesh;
}

function tileNoise(x, y, period, seed) {
  const xi = Math.floor(x);
  const yi = Math.floor(y);
  const xf = x - xi;
  const yf = y - yi;
  const u = xf * xf * (3 - 2 * xf);
  const v = yf * yf * (3 - 2 * yf);
  const h = (a, b) => {
    let n = Math.imul((a % period + period) % period + 31, 374761393) ^
            Math.imul((b % period + period) % period + 17, 668265263) ^ seed;
    n = Math.imul(n ^ (n >>> 13), 1274126177);
    return ((n ^ (n >>> 16)) >>> 0) / 4294967295;
  };
  const a = h(xi, yi);
  const b = h(xi + 1, yi);
  const c = h(xi, yi + 1);
  const d = h(xi + 1, yi + 1);
  return (a * (1 - u) + b * u) * (1 - v) + (c * (1 - u) + d * u) * v;
}

function makeWaterNormals(size = 512) {
  const data = new Uint8Array(size * size * 4);
  const height = new Float32Array(size * size);
  for (let y = 0; y < size; y++) {
    for (let x = 0; x < size; x++) {
      let v = 0;
      let amp = 0.55;
      for (let o = 0; o < 5; o++) {
        const period = 5 << o;
        v += tileNoise(x / size * period, y / size * period, period, 900 + o) * amp;
        amp *= 0.52;
      }
      height[y * size + x] = v;
    }
  }
  for (let y = 0; y < size; y++) {
    for (let x = 0; x < size; x++) {
      const l = height[y * size + ((x - 1 + size) % size)];
      const r = height[y * size + ((x + 1) % size)];
      const d = height[((y - 1 + size) % size) * size + x];
      const u = height[((y + 1) % size) * size + x];
      let nx = (l - r) * 2.8;
      let ny = (d - u) * 2.8;
      let nz = 1;
      const len = Math.hypot(nx, ny, nz);
      nx /= len; ny /= len; nz /= len;
      const i = (y * size + x) * 4;
      data[i] = Math.round((nx * 0.5 + 0.5) * 255);
      data[i + 1] = Math.round((ny * 0.5 + 0.5) * 255);
      data[i + 2] = Math.round((nz * 0.5 + 0.5) * 255);
      data[i + 3] = 255;
    }
  }
  const tex = new THREE.DataTexture(data, size, size, THREE.RGBAFormat);
  tex.wrapS = tex.wrapT = THREE.RepeatWrapping;
  tex.magFilter = THREE.LinearFilter;
  tex.minFilter = THREE.LinearMipmapLinearFilter;
  tex.generateMipmaps = true;
  tex.needsUpdate = true;
  return tex;
}

function makeShoreFoam() {
  const N = 400;
  const radius = new Float32Array(N);
  for (let i = 0; i < N; i++) {
    const a = (i / N) * Math.PI * 2;
    let lo = 55;
    let hi = 360;
    for (let k = 0; k < 26; k++) {
      const mid = (lo + hi) / 2;
      if (terrainHeight(Math.cos(a) * mid, Math.sin(a) * mid) > 0.06) lo = mid;
      else hi = mid;
    }
    radius[i] = lo;
  }
  const seg = N + 1;
  const pos = new Float32Array(seg * 2 * 3);
  const uv = new Float32Array(seg * 2 * 2);
  const idx = [];
  for (let i = 0; i <= N; i++) {
    const k = i % N;
    const a = (k / N) * Math.PI * 2;
    const ca = Math.cos(a);
    const sa = Math.sin(a);
    const r0 = radius[k] - 2.6;
    const r1 = radius[k] + 1.1;
    const vi = i * 2;
    pos[vi * 3] = ca * r0;
    pos[vi * 3 + 1] = 0.2;
    pos[vi * 3 + 2] = sa * r0;
    pos[(vi + 1) * 3] = ca * r1;
    pos[(vi + 1) * 3 + 1] = 0.24;
    pos[(vi + 1) * 3 + 2] = sa * r1;
    uv[vi * 2] = i / N;
    uv[vi * 2 + 1] = 0;
    uv[(vi + 1) * 2] = i / N;
    uv[(vi + 1) * 2 + 1] = 1;
    if (i < N) {
      const a2 = i * 2;
      idx.push(a2, a2 + 1, a2 + 2, a2 + 1, a2 + 3, a2 + 2);
    }
  }
  const geo = new THREE.BufferGeometry();
  geo.setAttribute('position', new THREE.BufferAttribute(pos, 3));
  geo.setAttribute('uv', new THREE.BufferAttribute(uv, 2));
  geo.setIndex(idx);
  const mat = new THREE.ShaderMaterial({
    uniforms: {
      uTime: { value: 0 },
    },
    vertexShader: /* glsl */ `
      varying vec2 vUv;
      void main() {
        vUv = uv;
        gl_Position = projectionMatrix * modelViewMatrix * vec4(position, 1.0);
      }`,
    fragmentShader: /* glsl */ `
      uniform float uTime;
      varying vec2 vUv;
      void main() {
        float wave = sin(vUv.x * 62.0 + uTime * 1.8) * 0.5 + 0.5;
        float spark = sin(vUv.x * 23.0 - uTime * 3.2 + sin(vUv.x * 97.0) * 2.0) * 0.5 + 0.5;
        float a = (0.3 + 0.34 * wave * spark) * (1.0 - vUv.y * 0.75);
        if (a < 0.02) discard;
        gl_FragColor = vec4(0.94, 0.97, 0.92, a);
      }`,
    transparent: true,
    depthWrite: false,
    side: THREE.DoubleSide,
    fog: false,
  });
  const mesh = new THREE.Mesh(geo, mat);
  mesh.renderOrder = 6;
  mesh.frustumCulled = false;
  return mesh;
}

function makeGlowTexture() {
  const c = document.createElement('canvas');
  c.width = c.height = 128;
  const ctx = c.getContext('2d');
  const g = ctx.createRadialGradient(64, 64, 0, 64, 64, 64);
  g.addColorStop(0, 'rgba(230,240,255,1)');
  g.addColorStop(0.18, 'rgba(200,220,255,0.55)');
  g.addColorStop(0.5, 'rgba(140,170,230,0.12)');
  g.addColorStop(1, 'rgba(80,110,180,0)');
  ctx.fillStyle = g;
  ctx.fillRect(0, 0, 128, 128);
  const tex = new THREE.CanvasTexture(c);
  tex.colorSpace = THREE.SRGBColorSpace;
  return tex;
}

export function createEnvironment(scene, renderer, startHour = 16.4) {
  const sky = new Sky();
  sky.scale.setScalar(9000);
  sky.renderOrder = -1000;
  sky.material.uniforms.turbidity.value = 8;
  sky.material.uniforms.rayleigh.value = 2.2;
  sky.material.uniforms.mieCoefficient.value = 0.005;
  sky.material.uniforms.mieDirectionalG.value = 0.8;
  scene.add(sky);

  const sunDir = new THREE.Vector3();
  const sunLight = new THREE.DirectionalLight(0xffffff, 3.0);
  sunLight.castShadow = true;
  sunLight.shadow.mapSize.set(2048, 2048);
  sunLight.shadow.camera.near = 30;
  sunLight.shadow.camera.far = 1600;
  sunLight.shadow.camera.left = -290;
  sunLight.shadow.camera.right = 290;
  sunLight.shadow.camera.top = 290;
  sunLight.shadow.camera.bottom = -290;
  sunLight.shadow.bias = -0.0006;
  sunLight.shadow.normalBias = 0.6;
  sunLight.target.position.set(0, 0, 0);
  scene.add(sunLight, sunLight.target);

  const moonLight = new THREE.DirectionalLight(0x9db6e8, 0.0);
  moonLight.position.set(0, 300, 0);
  scene.add(moonLight);

  const hemi = new THREE.HemisphereLight(0xbcd8ff, 0x2c2a18, 0.65);
  scene.add(hemi);
  const ambient = new THREE.AmbientLight(0x223044, 0.16);
  scene.add(ambient);

  const fill = new THREE.DirectionalLight(0xaecbd8, 0.0);
  fill.position.set(150, 520, 300);
  fill.target.position.set(0, 0, 0);
  scene.add(fill, fill.target);

  // ---- 星空 ----
  const starCount = 1800;
  const starPos = new Float32Array(starCount * 3);
  const starRng = mulberry32(99);
  for (let i = 0; i < starCount; i++) {
    const u = starRng() * Math.PI * 2;
    const v = Math.acos(1 - starRng() * 0.92);
    const r = 5600;
    starPos[i * 3] = Math.sin(v) * Math.cos(u) * r;
    starPos[i * 3 + 1] = Math.cos(v) * r;
    starPos[i * 3 + 2] = Math.sin(v) * Math.sin(u) * r;
  }
  const starGeo = new THREE.BufferGeometry();
  starGeo.setAttribute('position', new THREE.BufferAttribute(starPos, 3));
  const starMat = new THREE.PointsMaterial({
    color: 0xcfe2ff,
    size: 2.0,
    sizeAttenuation: false,
    transparent: true,
    opacity: 0,
    depthWrite: false,
    fog: false,
  });
  const stars = new THREE.Points(starGeo, starMat);
  stars.renderOrder = -970;
  scene.add(stars);

  // ---- 月亮 ----
  const moonGroup = new THREE.Group();
  const moon = new THREE.Mesh(
    new THREE.SphereGeometry(28, 20, 16),
    new THREE.MeshBasicMaterial({ color: 0xdfe8ff, fog: false, depthWrite: false })
  );
  const moonGlow = new THREE.Sprite(new THREE.SpriteMaterial({
    map: makeGlowTexture(),
    color: 0xbcd4ff,
    transparent: true,
    opacity: 0.8,
    depthWrite: false,
    depthTest: true,
    blending: THREE.AdditiveBlending,
    fog: false,
  }));
  moonGlow.scale.setScalar(260);
  moonGroup.add(moon, moonGlow);
  moonGroup.renderOrder = -960;
  scene.add(moonGroup);

  // ---- 云 ----
  const clouds = [makeCloudDome()];
  scene.add(...clouds);

  // ---- 湖面 ----
  const waterNormals = makeWaterNormals();
  const water = new Water(new THREE.PlaneGeometry(640, 640, 1, 1), {
    textureWidth: 256,
    textureHeight: 256,
    waterNormals,
    sunDirection: new THREE.Vector3(0.3, 0.8, 0.2).normalize(),
    sunColor: 0xfff2dd,
    waterColor: 0x0b2c31,
    distortionScale: 3.0,
    fog: true,
    alpha: 1.0,
  });
  water.material.fragmentShader = water.material.fragmentShader
    .replace('float rf0 = 0.3;', 'float rf0 = 0.035;')
    .replace('vec3( 0.1 )', 'vec3( 0.008 )');
  water.material.needsUpdate = true;
  water.rotation.x = -Math.PI / 2;
  water.position.y = 0.06;
  water.material.uniforms.size.value = 0.2;
  scene.add(water);

  const foam = makeShoreFoam();
  scene.add(foam);

  scene.fog = new THREE.FogExp2(0x92a9b8, 0.00036);

  // ---- 色彩预设 ----
  const dayFog = new THREE.Color(0x9fb8c4);
  const duskFog = new THREE.Color(0xb98063);
  const nightFog = new THREE.Color(0x050c15);
  const sunHigh = new THREE.Color(0xfff4e0);
  const sunLow = new THREE.Color(0xff9a4d);
  const hemiSkyDay = new THREE.Color(0xa8ccff);
  const hemiSkyDusk = new THREE.Color(0xffb07a);
  const hemiSkyNight = new THREE.Color(0x263a66);
  const cloudDay = new THREE.Color(0xf7f4ea);
  const cloudDusk = new THREE.Color(0xffbd88);
  const cloudNight = new THREE.Color(0x1b2740);
  const cloudShadowDay = new THREE.Color(0x8c9cab);
  const cloudShadowDusk = new THREE.Color(0x65505e);
  const cloudShadowNight = new THREE.Color(0x08111e);
  const waterDay = new THREE.Color(0x0d343a);
  const waterDusk = new THREE.Color(0x20292d);
  const waterNight = new THREE.Color(0x030b12);

  const state = {
    hours: startHour,
    sunDir,
    sunLight,
    moonLight,
    hemi,
    ambient,
    sky,
    water,
    stars,
    moonGroup,
    clouds,
    fogColor: new THREE.Color(),
    sunElevation: 0,
    isNight: false,
  };

  const tmpLit = new THREE.Color();
  const tmpShadow = new THREE.Color();
  const tmpWater = new THREE.Color();
  const tmpMoon = new THREE.Vector3();

  function setTime(hours) {
    state.hours = ((hours % 24) + 24) % 24;
    const t = ((state.hours - 6) / 12) * Math.PI;
    sunDir.set(Math.cos(t), Math.sin(t), 0.36).normalize();
    const elev = sunDir.y;
    state.sunElevation = elev;

    const twilight = Math.max(0, 1 - Math.abs(elev) / 0.22);
    const day = Math.max(0, Math.min(1, (elev + 0.04) / 0.42));
    const daySmooth = day * day * (3 - 2 * day);
    const night = 1 - Math.max(0, Math.min(1, (elev + 0.08) / 0.14));
    state.isNight = elev < -0.04;

    sky.material.uniforms.sunPosition.value.copy(sunDir);
    sky.material.uniforms.turbidity.value = 3.4 + 7.5 * (1 - daySmooth);
    sky.material.uniforms.rayleigh.value = 1.15 + 2.1 * (1 - daySmooth);
    sky.material.uniforms.mieCoefficient.value = 0.0028 + 0.0045 * (1 - daySmooth);

    sunLight.position.copy(sunDir).multiplyScalar(700);
    sunLight.color.copy(sunLow).lerp(sunHigh, THREE.MathUtils.smoothstep(elev, 0.02, 0.6));
    sunLight.intensity = 0.03 + 2.45 * THREE.MathUtils.smoothstep(elev, -0.05, 0.45);
    sunLight.shadow.intensity = 0.7 + 0.2 * daySmooth;

    moonLight.position.copy(sunDir).multiplyScalar(-700);
    moonLight.position.y = Math.max(120, moonLight.position.y);
    moonLight.intensity = 0.18 * night;

    hemi.color.copy(hemiSkyNight).lerp(hemiSkyDusk, twilight).lerp(hemiSkyDay, daySmooth);
    hemi.groundColor.setHex(0x33301d);
    hemi.intensity = 0.2 + 0.66 * daySmooth + 0.1 * twilight;
    ambient.intensity = 0.09 + 0.12 * daySmooth;
    fill.intensity = 0.08 + 0.62 * daySmooth;
    fill.color.copy(hemiSkyNight).lerp(hemiSkyDusk, twilight).lerp(hemiSkyDay, daySmooth);

    state.fogColor.copy(nightFog).lerp(duskFog, twilight * 0.8).lerp(dayFog, daySmooth * 0.9);
    scene.fog.color.copy(state.fogColor);
    scene.fog.density = 0.00062 - 0.00022 * daySmooth;

    water.material.uniforms.sunDirection.value.copy(sunDir);
    water.material.uniforms.sunColor.value.copy(sunLow).lerp(sunHigh, daySmooth);
    tmpWater.copy(waterNight).lerp(waterDusk, twilight).lerp(waterDay, daySmooth);
    water.material.uniforms.waterColor.value.copy(tmpWater);

    starMat.opacity = Math.pow(night, 1.6) * 0.85;
    stars.visible = starMat.opacity > 0.01;

    tmpMoon.copy(sunDir).multiplyScalar(-1);
    tmpMoon.y = Math.abs(tmpMoon.y) * 0.9 + 0.15;
    tmpMoon.normalize();
    moonGroup.position.copy(tmpMoon).multiplyScalar(4600);
    moonGroup.lookAt(0, 0, 0);
    moonGroup.visible = night > 0.05;

    for (let i = 0; i < clouds.length; i++) {
      const mat = clouds[i].material;
      mat.uniforms.uSun.value = 0.35 + 0.65 * daySmooth;
      tmpLit.copy(cloudNight).lerp(cloudDusk, twilight).lerp(cloudDay, daySmooth);
      tmpShadow.copy(cloudShadowNight).lerp(cloudShadowDusk, twilight).lerp(cloudShadowDay, daySmooth);
      mat.uniforms.uLit.value.copy(tmpLit);
      mat.uniforms.uShadow.value.copy(tmpShadow);
      mat.uniforms.uSunDir.value.copy(sunDir);
      mat.uniforms.uOpacity.value = 0.06 + 0.16 * daySmooth + 0.04 * twilight;
      mat.uniforms.uCoverage.value = 0.5 + 0.05 * (1 - daySmooth);
    }

    scene.environmentIntensity = 0.06 + 0.32 * daySmooth;
    renderer.toneMappingExposure = 0.63 + 0.18 * daySmooth;
  }

  setTime(startHour);

  // 用当前天空生成一次环境光照，让石材、树叶有环境反射
  const envScene = new THREE.Scene();
  const envSky = new Sky();
  envSky.scale.setScalar(800);
  envSky.material.uniforms.turbidity.value = sky.material.uniforms.turbidity.value;
  envSky.material.uniforms.rayleigh.value = sky.material.uniforms.rayleigh.value;
  envSky.material.uniforms.mieCoefficient.value = sky.material.uniforms.mieCoefficient.value;
  envSky.material.uniforms.mieDirectionalG.value = 0.8;
  envSky.material.uniforms.sunPosition.value.copy(sunDir);
  envScene.add(envSky);
  const pmrem = new THREE.PMREMGenerator(renderer);
  const envRT = pmrem.fromScene(envScene, 0, 0.1, 1200);
  scene.environment = envRT.texture;
  scene.environmentIntensity = 0.55;
  pmrem.dispose();

  state.setTime = setTime;
  state.update = (dt, elapsed) => {
    water.material.uniforms.time.value += dt * 0.55;
    for (const c of clouds) c.material.uniforms.uTime.value = elapsed;
    stars.rotation.y = elapsed * 0.002;
    foam.material.uniforms.uTime.value = elapsed;
  };

  return state;
}
