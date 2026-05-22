#pragma once

#include "../EUINEO.h"
#include <string>
#include <vector>

namespace EUINEO {

struct CppDrawCommand {
    std::string type;         // "FilledRect", "Text", "Polygon", "RectOutline"
    float rect[4];            // x, y, w, h
    float clipRect[4];        // scissor clip area
    float color[4];           // r, g, b, a
    float radius;             // rounding
    float thickness;          // border width
    float rotation;
    float blurRadius;
    float effectAlpha;        // opacity
    bool hasClip;
    std::string text;
    float fontSize;
    bool hasGradient;
    float gradientColors[16]; // 4 corners * RGBA
    float shadowBlur;
    float shadowOffset[2];
    float shadowColor[4];
};

namespace DumpState {
    void BeginRecording();
    void EndRecordingAndWrite(const char* path);
    bool IsRecording();
    void RecordRect(float x, float y, float w, float h, const RectStyle& style);
    void RecordText(const std::string& text, float x, float y, const Color& color, float scale);
    void RecordPolygon(const std::vector<Point2>& points, const Color& fill, const RectGradient& gradient, float strokeWidth, const Color& stroke);
}

} // namespace EUINEO
