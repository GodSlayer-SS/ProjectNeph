import { useEffect, useRef } from "react";
import { useOrbStore } from "../state/orbStore";

/**
 * OrbCanvas — Nephis Orb v1
 *
 * A 200-LOC WebGL2 ring that:
 *   - Pulses with audio amplitude (voice:amplitude events)
 *   - Changes colour across 5 states (idle/listening/thinking/speaking/error)
 *   - Runs at 60fps via requestAnimationFrame
 *
 * Deliberately simple. No R3F, no WebGPU, no particles.
 * Orb v2 (WebGPU + R3F) is Phase 4.
 */

const STATE_COLORS: Record<string, [number, number, number]> = {
  idle:          [0.38, 0.18, 0.72],   // deep violet
  listening:     [0.08, 0.52, 0.98],   // electric blue
  transcribing:  [0.08, 0.82, 0.92],   // cyan — processing audio
  thinking:      [0.98, 0.68, 0.08],   // amber
  speaking:      [0.12, 0.82, 0.42],   // emerald green
  error:         [0.95, 0.22, 0.22],   // red
};

const VERT_SRC = `#version 300 es
precision highp float;
in vec2 a_pos;
out vec2 v_uv;
void main() {
  v_uv = a_pos * 0.5 + 0.5;
  gl_Position = vec4(a_pos, 0.0, 1.0);
}`;

const FRAG_SRC = `#version 300 es
precision highp float;
in vec2 v_uv;
out vec4 fragColor;

uniform float u_time;
uniform float u_amplitude;
uniform vec3  u_color;

float ring(vec2 uv, float r, float thickness) {
  float d = length(uv - 0.5);
  return smoothstep(thickness, 0.0, abs(d - r));
}

void main() {
  // Amplitude-reactive ring radius
  float amp = 0.12 + u_amplitude * 0.10;
  float pulse = amp + sin(u_time * 3.0) * 0.012;
  float r = ring(v_uv, pulse, 0.014);

  // Soft inner glow
  float glow_r = length(v_uv - 0.5);
  float glow = exp(-glow_r * 12.0) * 0.35 * (0.5 + u_amplitude);

  float alpha = clamp(r + glow, 0.0, 1.0);
  fragColor = vec4(u_color * alpha, alpha);
}`;

export function OrbCanvas() {
  const canvasRef = useRef<HTMLCanvasElement>(null);
  const { state: orbState, amplitude } = useOrbStore();

  useEffect(() => {
    const canvas = canvasRef.current;
    if (!canvas) return;
    const gl = canvas.getContext("webgl2", { premultipliedAlpha: false });
    if (!gl) return;

    const compile = (type: number, src: string) => {
      const s = gl.createShader(type)!;
      gl.shaderSource(s, src);
      gl.compileShader(s);
      return s;
    };

    const prog = gl.createProgram()!;
    gl.attachShader(prog, compile(gl.VERTEX_SHADER, VERT_SRC));
    gl.attachShader(prog, compile(gl.FRAGMENT_SHADER, FRAG_SRC));
    gl.linkProgram(prog);
    gl.useProgram(prog);

    // Full-screen quad
    const buf = gl.createBuffer();
    gl.bindBuffer(gl.ARRAY_BUFFER, buf);
    gl.bufferData(
      gl.ARRAY_BUFFER,
      new Float32Array([-1, -1, 1, -1, -1, 1, 1, 1]),
      gl.STATIC_DRAW,
    );
    const loc = gl.getAttribLocation(prog, "a_pos");
    gl.enableVertexAttribArray(loc);
    gl.vertexAttribPointer(loc, 2, gl.FLOAT, false, 0, 0);

    const uTime = gl.getUniformLocation(prog, "u_time");
    const uAmp  = gl.getUniformLocation(prog, "u_amplitude");
    const uColor = gl.getUniformLocation(prog, "u_color");

    gl.enable(gl.BLEND);
    gl.blendFunc(gl.SRC_ALPHA, gl.ONE_MINUS_SRC_ALPHA);

    let rafId: number;
    const render = (t: number) => {
      const color = STATE_COLORS[orbState] ?? STATE_COLORS.idle;
      gl.viewport(0, 0, canvas.width, canvas.height);
      gl.clearColor(0, 0, 0, 0);
      gl.clear(gl.COLOR_BUFFER_BIT);
      gl.uniform1f(uTime, t / 1000);
      gl.uniform1f(uAmp, amplitude);
      gl.uniform3f(uColor, color[0], color[1], color[2]);
      gl.drawArrays(gl.TRIANGLE_STRIP, 0, 4);
      rafId = requestAnimationFrame(render);
    };
    rafId = requestAnimationFrame(render);

    return () => {
      cancelAnimationFrame(rafId);
      gl.deleteProgram(prog);
      gl.deleteBuffer(buf);
    };
  }, [orbState, amplitude]);

  return (
    <canvas
      ref={canvasRef}
      width={200}
      height={200}
      id="orb-canvas"
      aria-label={`Nephis orb — ${orbState}. Hold Alt+V to talk.`}
      title="Hold Alt+V to talk"
      style={{ display: "block", margin: "0 auto", cursor: "default" }}
    />
  );
}
