#version 450
#extension GL_ARB_separate_shader_objects : enable

layout(location = 0) in vec4 inColor;
layout(location = 1) in vec2 inTexCoords;
layout(location = 0) out vec4 outColor;
layout(set = 0, binding = 0) uniform texture2D uColorTex;
layout(set = 0, binding = 1) uniform sampler uColorTexSampler;

void main() {
    vec4 texColor = texture(sampler2D(uColorTex, uColorTexSampler), inTexCoords);
    outColor = inColor * texColor;
}
