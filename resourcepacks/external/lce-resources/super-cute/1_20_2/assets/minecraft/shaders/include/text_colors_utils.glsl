#version 150

#moj_import <color_overrides.glsl>

uniform float GameTime;

precision highp float;

#define finalize() { \
    vertexDistance=length((ModelViewMat*vertex).xyz); \
    texCoord0=UV0; \
}

void applyProjection(inout vec4 vertex) {
    gl_Position = ProjMat * ModelViewMat * vertex;
}

void applyColorTexture() {
    vertexColor = Color * texelFetch(Sampler2, UV2 / 16, 0);
}

void processColorReplace(inout vec4 vertex, vec3 newColor) {
    applyProjection(vertex);
    vertexColor = vec4(newColor, Color.a) * texelFetch(Sampler2, UV2 / 16, 0);
    finalize();
}

void applyTextColors() {
    vec4 vertex = vec4(Position, 1.);
    ivec3 iColor = ivec3(Color.xyz * 255 + vec3(.5));

    // Standard Colors
    if (iColor == ivec3(0, 0, 0)) { processColorReplace(vertex, black); return; }
    if (iColor == ivec3(0, 0, 170)) { processColorReplace(vertex, dark_blue); return; }
    if (iColor == ivec3(0, 170, 0)) { processColorReplace(vertex, dark_green); return; }
    if (iColor == ivec3(0, 170, 170)) { processColorReplace(vertex, dark_aqua); return; }
    if (iColor == ivec3(170, 0, 0)) { processColorReplace(vertex, dark_red); return; }
    if (iColor == ivec3(170, 0, 170)) { processColorReplace(vertex, dark_purple); return; }
    if (iColor == ivec3(255, 170, 0)) { processColorReplace(vertex, gold); return; }
    if (iColor == ivec3(170, 170, 170)) { processColorReplace(vertex, gray); return; }
    if (iColor == ivec3(85, 85, 85)) { processColorReplace(vertex, dark_gray); return; }
    if (iColor == ivec3(85, 85, 255)) { processColorReplace(vertex, blue); return; }
    if (iColor == ivec3(85, 255, 85)) { processColorReplace(vertex, green); return; }
    if (iColor == ivec3(85, 255, 255)) { processColorReplace(vertex, aqua); return; }
    if (iColor == ivec3(255, 85, 85)) { processColorReplace(vertex, red); return; }
    if (iColor == ivec3(255, 85, 255)) { processColorReplace(vertex, light_purple); return; }
    if (iColor == ivec3(255, 255, 85)) { processColorReplace(vertex, yellow); return; }
    if (iColor == ivec3(255, 255, 255)) { processColorReplace(vertex, white); return; }
    if (iColor == ivec3(64, 64, 64)) { processColorReplace(vertex, title_text_color); return; }
    if (iColor == ivec3(224, 224, 224)) { processColorReplace(vertex, body_text_color); return; }
    if (iColor == ivec3(128, 255, 32)) { processColorReplace(vertex, experience_text_color); return; }

    // Shadow Colors (Standard / 4)
    if (iColor == ivec3(0, 0, 42)) { processColorReplace(vertex, dark_blue * 0.25); return; }
    if (iColor == ivec3(0, 42, 0)) { processColorReplace(vertex, dark_green * 0.25); return; }
    if (iColor == ivec3(0, 42, 42)) { processColorReplace(vertex, dark_aqua * 0.25); return; }
    if (iColor == ivec3(42, 0, 0)) { processColorReplace(vertex, dark_red * 0.25); return; }
    if (iColor == ivec3(42, 0, 42)) { processColorReplace(vertex, dark_purple * 0.25); return; }
    if (iColor == ivec3(63, 42, 0)) { processColorReplace(vertex, gold * 0.25); return; }
    if (iColor == ivec3(42, 42, 42)) { processColorReplace(vertex, gray * 0.25); return; }
    if (iColor == ivec3(21, 21, 21)) { processColorReplace(vertex, dark_gray * 0.25); return; }
    if (iColor == ivec3(21, 21, 63)) { processColorReplace(vertex, blue * 0.25); return; }
    if (iColor == ivec3(21, 63, 21)) { processColorReplace(vertex, green * 0.25); return; }
    if (iColor == ivec3(21, 63, 63)) { processColorReplace(vertex, aqua * 0.25); return; }
    if (iColor == ivec3(63, 21, 21)) { processColorReplace(vertex, red * 0.25); return; }
    if (iColor == ivec3(63, 21, 63)) { processColorReplace(vertex, light_purple * 0.25); return; }
    if (iColor == ivec3(63, 63, 21)) { processColorReplace(vertex, yellow * 0.25); return; }
    if (iColor == ivec3(63, 63, 63)) { processColorReplace(vertex, white * 0.25); return; }
    if (iColor == ivec3(16, 16, 16)) { processColorReplace(vertex, title_text_color * 0.25); return; }
    if (iColor == ivec3(56, 56, 56)) { processColorReplace(vertex, body_text_color * 0.25); return; }
    if (iColor == ivec3(32, 63, 8)) { processColorReplace(vertex, experience_text_color * 0.25); return; }

    applyProjection(vertex);
    applyColorTexture();
    finalize();
}
