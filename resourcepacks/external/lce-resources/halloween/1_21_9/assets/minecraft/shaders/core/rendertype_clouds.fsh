#version 330

#moj_import <minecraft:fog.glsl>


in float vertexDistance;
in vec4 vertexColor;

out vec4 fragColor;

void main() {
    vec4 color = vertexColor;
    // Override cloud RGB to hex #2B0845 -> RGB(43, 8, 69)
    vec3 overrideCloud = vec3(43.0/255.0, 8.0/255.0, 69.0/255.0);
    color.rgb = overrideCloud;
    // Preserve alpha but fade with fog as before
    color.a *= 1.0f - linear_fog_value(vertexDistance, 0, FogCloudsEnd);
    fragColor = color;
}
