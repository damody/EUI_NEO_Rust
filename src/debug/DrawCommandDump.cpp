#include "DrawCommandDump.h"
#include <cstdio>
#include <cstring>

namespace EUINEO {

static bool sRecording = false;
static std::vector<CppDrawCommand> sCommands;

void DumpState::BeginRecording() {
    sRecording = true;
    sCommands.clear();
}

bool DumpState::IsRecording() {
    return sRecording;
}

void DumpState::RecordRect(float x, float y, float w, float h, const RectStyle& style) {
    CppDrawCommand cmd{};
    cmd.type = "FilledRect";
    cmd.rect[0] = x; cmd.rect[1] = y; cmd.rect[2] = w; cmd.rect[3] = h;
    cmd.color[0] = style.color.r; cmd.color[1] = style.color.g;
    cmd.color[2] = style.color.b; cmd.color[3] = style.color.a;
    cmd.radius = style.rounding;
    cmd.blurRadius = style.blurAmount;
    cmd.rotation = style.transform.rotationDegrees;
    cmd.effectAlpha = style.color.a;
    cmd.hasGradient = style.gradient.enabled;
    if (style.gradient.enabled) {
        const Color* corners[4] = {&style.gradient.topLeft, &style.gradient.topRight,
                                    &style.gradient.bottomLeft, &style.gradient.bottomRight};
        for (int i = 0; i < 4; ++i) {
            cmd.gradientColors[i * 4 + 0] = corners[i]->r;
            cmd.gradientColors[i * 4 + 1] = corners[i]->g;
            cmd.gradientColors[i * 4 + 2] = corners[i]->b;
            cmd.gradientColors[i * 4 + 3] = corners[i]->a;
        }
    }
    cmd.shadowBlur = style.shadowBlur;
    cmd.shadowOffset[0] = style.shadowOffsetX;
    cmd.shadowOffset[1] = style.shadowOffsetY;
    cmd.shadowColor[0] = style.shadowColor.r;
    cmd.shadowColor[1] = style.shadowColor.g;
    cmd.shadowColor[2] = style.shadowColor.b;
    cmd.shadowColor[3] = style.shadowColor.a;
    sCommands.push_back(std::move(cmd));
}

void DumpState::RecordText(const std::string& text, float x, float y, const Color& color, float scale) {
    CppDrawCommand cmd{};
    cmd.type = "Text";
    cmd.rect[0] = x; cmd.rect[1] = y; cmd.rect[2] = 0.0f; cmd.rect[3] = 0.0f;
    cmd.color[0] = color.r; cmd.color[1] = color.g;
    cmd.color[2] = color.b; cmd.color[3] = color.a;
    cmd.text = text;
    cmd.fontSize = scale;
    cmd.effectAlpha = color.a;
    sCommands.push_back(std::move(cmd));
}

void DumpState::RecordPolygon(const std::vector<Point2>& points, const Color& fill,
                               const RectGradient& gradient, float strokeWidth, const Color& stroke) {
    CppDrawCommand cmd{};
    cmd.type = "Polygon";
    if (!points.empty()) {
        float minX = points[0].x, minY = points[0].y;
        float maxX = points[0].x, maxY = points[0].y;
        for (const auto& p : points) {
            if (p.x < minX) minX = p.x;
            if (p.y < minY) minY = p.y;
            if (p.x > maxX) maxX = p.x;
            if (p.y > maxY) maxY = p.y;
        }
        cmd.rect[0] = minX; cmd.rect[1] = minY;
        cmd.rect[2] = maxX - minX; cmd.rect[3] = maxY - minY;
    }
    cmd.color[0] = fill.r; cmd.color[1] = fill.g;
    cmd.color[2] = fill.b; cmd.color[3] = fill.a;
    cmd.thickness = strokeWidth;
    cmd.hasGradient = gradient.enabled;
    if (gradient.enabled) {
        const Color* corners[4] = {&gradient.topLeft, &gradient.topRight,
                                    &gradient.bottomLeft, &gradient.bottomRight};
        for (int i = 0; i < 4; ++i) {
            cmd.gradientColors[i * 4 + 0] = corners[i]->r;
            cmd.gradientColors[i * 4 + 1] = corners[i]->g;
            cmd.gradientColors[i * 4 + 2] = corners[i]->b;
            cmd.gradientColors[i * 4 + 3] = corners[i]->a;
        }
    }
    cmd.effectAlpha = fill.a;
    sCommands.push_back(std::move(cmd));
}

static void WriteF32(FILE* f, float v) {
    fprintf(f, "%.2f", v);
}

static void WriteF32Array(FILE* f, const float* arr, int count) {
    fprintf(f, "[");
    for (int i = 0; i < count; ++i) {
        if (i > 0) fprintf(f, ", ");
        WriteF32(f, arr[i]);
    }
    fprintf(f, "]");
}

static void WriteJsonString(FILE* f, const std::string& s) {
    fprintf(f, "\"");
    for (char ch : s) {
        switch (ch) {
            case '"': fprintf(f, "\\\""); break;
            case '\\': fprintf(f, "\\\\"); break;
            case '\n': fprintf(f, "\\n"); break;
            case '\r': fprintf(f, "\\r"); break;
            case '\t': fprintf(f, "\\t"); break;
            default:
                if (static_cast<unsigned char>(ch) < 0x20) {
                    fprintf(f, "\\u%04x", static_cast<unsigned int>(static_cast<unsigned char>(ch)));
                } else {
                    fprintf(f, "%c", ch);
                }
                break;
        }
    }
    fprintf(f, "\"");
}

void DumpState::EndRecordingAndWrite(const char* path) {
    sRecording = false;
    FILE* f = fopen(path, "w");
    if (!f) {
        sCommands.clear();
        return;
    }

    fprintf(f, "{\n");
    fprintf(f, "  \"frame_command_count\": %d,\n", static_cast<int>(sCommands.size()));
    fprintf(f, "  \"commands\": [\n");

    for (size_t i = 0; i < sCommands.size(); ++i) {
        const CppDrawCommand& cmd = sCommands[i];
        fprintf(f, "    {\n");
        fprintf(f, "      \"index\": %d,\n", static_cast<int>(i));
        fprintf(f, "      \"type\": \"%s\",\n", cmd.type.c_str());
        fprintf(f, "      \"rect\": "); WriteF32Array(f, cmd.rect, 4); fprintf(f, ",\n");
        fprintf(f, "      \"clip_rect\": "); WriteF32Array(f, cmd.clipRect, 4); fprintf(f, ",\n");
        fprintf(f, "      \"color\": "); WriteF32Array(f, cmd.color, 4); fprintf(f, ",\n");
        fprintf(f, "      \"radius\": "); WriteF32(f, cmd.radius); fprintf(f, ",\n");
        fprintf(f, "      \"blur_radius\": "); WriteF32(f, cmd.blurRadius); fprintf(f, ",\n");
        fprintf(f, "      \"effect_alpha\": "); WriteF32(f, cmd.effectAlpha); fprintf(f, ",\n");
        fprintf(f, "      \"has_clip\": %s,\n", cmd.hasClip ? "true" : "false");
        fprintf(f, "      \"text\": "); WriteJsonString(f, cmd.text); fprintf(f, ",\n");
        fprintf(f, "      \"font_size\": "); WriteF32(f, cmd.fontSize); fprintf(f, ",\n");

        if (cmd.hasGradient) {
            fprintf(f, "      \"gradient\": {\n");
            fprintf(f, "        \"top_left\": "); WriteF32Array(f, &cmd.gradientColors[0], 4); fprintf(f, ",\n");
            fprintf(f, "        \"top_right\": "); WriteF32Array(f, &cmd.gradientColors[4], 4); fprintf(f, ",\n");
            fprintf(f, "        \"bottom_left\": "); WriteF32Array(f, &cmd.gradientColors[8], 4); fprintf(f, ",\n");
            fprintf(f, "        \"bottom_right\": "); WriteF32Array(f, &cmd.gradientColors[12], 4); fprintf(f, "\n");
            fprintf(f, "      },\n");
        } else {
            fprintf(f, "      \"gradient\": null,\n");
        }

        if (cmd.shadowBlur > 0.0f || cmd.shadowColor[3] > 0.0f) {
            fprintf(f, "      \"shadow\": {\n");
            fprintf(f, "        \"blur\": "); WriteF32(f, cmd.shadowBlur); fprintf(f, ",\n");
            fprintf(f, "        \"offset\": "); WriteF32Array(f, cmd.shadowOffset, 2); fprintf(f, ",\n");
            fprintf(f, "        \"color\": "); WriteF32Array(f, cmd.shadowColor, 4); fprintf(f, "\n");
            fprintf(f, "      },\n");
        } else {
            fprintf(f, "      \"shadow\": null,\n");
        }

        fprintf(f, "      \"transform\": null\n");
        fprintf(f, "    }%s\n", (i + 1 < sCommands.size()) ? "," : "");
    }

    fprintf(f, "  ]\n");
    fprintf(f, "}\n");
    fclose(f);

    printf("[EUI-NEO] Dumped %d draw commands to %s\n", static_cast<int>(sCommands.size()), path);
    sCommands.clear();
}

} // namespace EUINEO
