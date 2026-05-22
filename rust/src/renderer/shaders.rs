/// Vertex shader for rounded-rect rendering (matches C++ EUINEO rect shader).
pub const RECT_VERTEX_SHADER: &str = r#"
#version 330 core
layout(location = 0) in vec2 aPos;
out vec2 vPos;
uniform mat4 projection;
uniform vec2 uPos;
uniform vec2 uSize;
void main() {
    vec2 pos = uPos + aPos * uSize;
    vPos = pos;
    gl_Position = projection * vec4(pos, 0.0, 1.0);
}
"#;

/// Fragment shader for rounded-rect rendering (matches C++ EUINEO rect shader).
pub const RECT_FRAGMENT_SHADER: &str = r#"
#version 330 core
in vec2 vPos;
uniform vec4 uColor;
uniform vec2 uBoxPos;
uniform vec2 uBoxSize;
uniform vec2 uTranslate;
uniform mat2 uTransformInv;
uniform float uRounding;
uniform float uBlurAmount;
uniform float uShadowBlur;
uniform vec2 uShadowOffset;
uniform vec4 uShadowColor;
uniform int uGradientEnabled;
uniform vec4 uGradientTopLeft;
uniform vec4 uGradientTopRight;
uniform vec4 uGradientBottomLeft;
uniform vec4 uGradientBottomRight;
uniform float iTime;
uniform vec2 iResolution;
uniform sampler2D iChannel0;
out vec4 FragColor;

float roundedBoxSDF(vec2 centerPosition, vec2 size, float radius) {
    return length(max(abs(centerPosition) - size + radius, 0.0)) - radius;
}

vec3 draw(vec2 uv) {
    return texture(iChannel0, uv).rgb;
}

float rand(vec2 co) {
    return fract(sin(dot(co.xy, vec2(12.9898, 78.233))) * 43758.5453);
}

vec4 fillColorAt(vec2 uv) {
    if (uGradientEnabled == 0) {
        return uColor;
    }
    vec4 top = mix(uGradientTopLeft, uGradientTopRight, clamp(uv.x, 0.0, 1.0));
    vec4 bottom = mix(uGradientBottomLeft, uGradientBottomRight, clamp(uv.x, 0.0, 1.0));
    vec4 gradientColor = mix(top, bottom, clamp(uv.y, 0.0, 1.0));
    return vec4(gradientColor.rgb, gradientColor.a * uColor.a);
}

void main() {
    vec2 center = uBoxPos + uBoxSize * 0.5 + uTranslate;
    vec2 p = uTransformInv * (vPos - center);
    float d = roundedBoxSDF(p, uBoxSize * 0.5, uRounding);

    float shadowAlpha = 0.0;
    if (uShadowBlur > 0.0) {
        vec2 shadowDelta = (vPos - center) - uShadowOffset;
        vec2 sp = uTransformInv * shadowDelta;
        float sd = roundedBoxSDF(sp, uBoxSize * 0.5, uRounding);
        shadowAlpha = 1.0 - smoothstep(-uShadowBlur, uShadowBlur, sd);
        shadowAlpha *= uShadowColor.a;
    }

    float alpha = 1.0 - smoothstep(-1.0, 1.0, d);
    vec2 safeBoxSize = max(uBoxSize, vec2(0.001));
    vec2 fillUV = clamp((p / safeBoxSize) + vec2(0.5), 0.0, 1.0);
    vec4 fillColor = fillColorAt(fillUV);
    vec4 finalColor = vec4(0.0);

    if (uBlurAmount > 0.0 && alpha > 0.0) {
        vec2 uv = gl_FragCoord.xy / iResolution.xy;
        float bluramount = uBlurAmount;
        vec2 pixelStep = 1.0 / iResolution.xy;
        float blurRadiusPx = bluramount * min(iResolution.x, iResolution.y);
        vec3 blurredImage = draw(uv);
        float repeats = mix(10.0, 28.0, clamp(bluramount / 0.15, 0.0, 1.0));
        const float tau = 6.28318530718;
        for (float i = 0.0; i < repeats; i += 1.0) {
            float angle = (i / repeats) * tau;
            vec2 dir = vec2(cos(angle), sin(angle));

            float radiusA = blurRadiusPx * (0.35 + 0.65 * rand(vec2(i, uv.x + uv.y)));
            vec2 uv2 = clamp(uv + dir * radiusA * pixelStep, pixelStep * 0.5, vec2(1.0) - pixelStep * 0.5);
            blurredImage += draw(uv2);

            float angleB = angle + (0.5 * tau / repeats);
            vec2 dirB = vec2(cos(angleB), sin(angleB));
            float radiusB = blurRadiusPx * (0.20 + 0.80 * rand(vec2(i + 2.0, uv.x + uv.y + 24.0)));
            uv2 = clamp(uv + dirB * radiusB * pixelStep, pixelStep * 0.5, vec2(1.0) - pixelStep * 0.5);
            blurredImage += draw(uv2);
        }
        blurredImage /= (repeats * 2.0 + 1.0);
        vec3 mixColor = mix(blurredImage, fillColor.rgb, fillColor.a);
        finalColor = vec4(mixColor, alpha);
    } else {
        finalColor = vec4(fillColor.rgb, fillColor.a * alpha);
    }

    if (shadowAlpha > 0.0 && alpha < 1.0) {
        vec3 outRgb = (finalColor.rgb * finalColor.a + uShadowColor.rgb * shadowAlpha * (1.0 - finalColor.a)) /
                      max(0.001, (finalColor.a + shadowAlpha * (1.0 - finalColor.a)));
        float outA = finalColor.a + shadowAlpha * (1.0 - finalColor.a);
        FragColor = vec4(outRgb, outA);
    } else {
        FragColor = finalColor;
    }
}
"#;

/// Cached blur vertex shader (matches C++ cachedBlurVShaderStr).
pub const CACHED_BLUR_VERTEX_SHADER: &str = r#"
#version 330 core
layout(location = 0) in vec2 aPos;
out vec2 vUV;
out vec2 vPos;
uniform mat4 projection;
uniform vec2 uPos;
uniform vec2 uSize;
void main() {
    vec2 pos = (aPos * uSize) + uPos;
    vUV = vec2(aPos.x, 1.0 - aPos.y);
    vPos = pos;
    gl_Position = projection * vec4(pos, 0.0, 1.0);
}
"#;

/// Cached blur fragment shader (matches C++ cachedBlurFShaderStr).
pub const CACHED_BLUR_FRAGMENT_SHADER: &str = r#"
#version 330 core
in vec2 vUV;
in vec2 vPos;
uniform sampler2D uTexture;
uniform vec2 uBoxPos;
uniform vec2 uBoxSize;
uniform vec2 uTranslate;
uniform mat2 uTransformInv;
uniform float uRounding;
uniform float uShadowBlur;
uniform vec2 uShadowOffset;
uniform float uShadowAlpha;
out vec4 FragColor;

float roundedBoxSDF(vec2 centerPosition, vec2 size, float radius) {
    return length(max(abs(centerPosition) - size + radius, 0.0)) - radius;
}

void main() {
    vec2 center = uBoxPos + uBoxSize * 0.5 + uTranslate;
    vec2 p = uTransformInv * (vPos - center);
    float d = roundedBoxSDF(p, uBoxSize * 0.5, uRounding);
    float alpha = 1.0 - smoothstep(-1.0, 1.0, d);

    float shadowAlpha = 0.0;
    if (uShadowBlur > 0.0 && uShadowAlpha > 0.0) {
        vec2 shadowDelta = (vPos - center) - uShadowOffset;
        vec2 sp = uTransformInv * shadowDelta;
        float sd = roundedBoxSDF(sp, uBoxSize * 0.5, uRounding);
        shadowAlpha = (1.0 - smoothstep(-uShadowBlur, uShadowBlur, sd)) * uShadowAlpha;
    }

    if (alpha <= 0.0 && shadowAlpha <= 0.0) {
        discard;
    }

    FragColor = texture(uTexture, vUV);
}
"#;

/// Composite vertex shader (matches C++ compositeVShaderStr).
pub const COMPOSITE_VERTEX_SHADER: &str = r#"
#version 330 core
layout(location = 0) in vec2 aPos;
layout(location = 1) in vec2 aUV;
out vec2 vUV;
uniform mat4 projection;
uniform vec2 uPos;
uniform vec2 uSize;
uniform vec2 uUVPos;
uniform vec2 uUVSize;
void main() {
    vec2 pos = uPos + aPos * uSize;
    vUV = uUVPos + aUV * uUVSize;
    gl_Position = projection * vec4(pos, 0.0, 1.0);
}
"#;

/// Composite fragment shader (matches C++ compositeFShaderStr).
pub const COMPOSITE_FRAGMENT_SHADER: &str = r#"
#version 330 core
in vec2 vUV;
uniform sampler2D uTexture;
out vec4 FragColor;
void main() {
    FragColor = texture(uTexture, vUV);
}
"#;

/// Polygon vertex shader (matches C++ polygonVShaderStr).
pub const POLYGON_VERTEX_SHADER: &str = r#"
#version 330 core
layout(location = 0) in vec2 aPos;
layout(location = 1) in vec2 aUV;
uniform mat4 projection;
out vec2 vUV;
void main() {
    vUV = aUV;
    gl_Position = projection * vec4(aPos, 0.0, 1.0);
}
"#;

/// Polygon fragment shader (matches C++ polygonFShaderStr).
pub const POLYGON_FRAGMENT_SHADER: &str = r#"
#version 330 core
uniform vec4 uColor;
uniform int uGradientEnabled;
uniform vec4 uGradientTopLeft;
uniform vec4 uGradientTopRight;
uniform vec4 uGradientBottomLeft;
uniform vec4 uGradientBottomRight;
in vec2 vUV;
out vec4 FragColor;
vec4 fillColorAt(vec2 uv) {
    vec4 top = mix(uGradientTopLeft, uGradientTopRight, clamp(uv.x, 0.0, 1.0));
    vec4 bottom = mix(uGradientBottomLeft, uGradientBottomRight, clamp(uv.x, 0.0, 1.0));
    return mix(top, bottom, clamp(uv.y, 0.0, 1.0));
}
void main() {
    FragColor = (uGradientEnabled == 1) ? fillColorAt(vUV) : uColor;
}
"#;

/// Text vertex shader (matches C++ textVShaderStr).
pub const TEXT_VERTEX_SHADER: &str = r#"
#version 330 core
layout(location = 0) in vec4 vertex;
out vec2 TexCoords;
uniform mat4 projection;
void main() {
    gl_Position = projection * vec4(vertex.xy, 0.0, 1.0);
    TexCoords = vertex.zw;
}
"#;

/// Text fragment shader (matches C++ textFShaderStr with SDF support).
pub const TEXT_FRAGMENT_SHADER: &str = r#"
#version 330 core
in vec2 TexCoords;
out vec4 color;
uniform sampler2D text;
uniform vec4 textColor;
uniform int textMode;
uniform float sdfEdgeValue;
uniform float sdfPxRange;
void main() {
    float sampleValue = texture(text, TexCoords).r;
    float alpha = sampleValue;
    if (textMode == 1) {
        float signedDistance = sampleValue - sdfEdgeValue;
        float valueSpread = max(fwidth(sampleValue), 0.0008);
        alpha = smoothstep(-valueSpread, valueSpread, signedDistance);
        if (alpha < 0.01) {
            discard;
        }
    }
    color = vec4(textColor.rgb, textColor.a * alpha);
}
"#;
