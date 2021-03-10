#version 450
#extension GL_ARB_separate_shader_objects : enable

layout(location = 0) in vec2 inPosition;
layout(location = 1) in vec2 inTexCoords;
layout(location = 0) out vec4 outColor;
layout(location = 1) out vec2 outTexCoords;
layout(push_constant) uniform PushConstant {
    mat4 transform;
    vec4 color;
} pushConstant;

void main() {
    gl_Position = pushConstant.transform * vec4(inPosition.x, inPosition.y, 0., 1.);
    outColor = pushConstant.color;
    outTexCoords = inTexCoords;
}