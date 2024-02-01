#include "emulatorthread.h"

#include <QImage>
#include <QPixmap>
#include "gg.hpp"

void EmulatorThread::run()
{
    size_t frame_size = INTERNAL_WIDTH * INTERNAL_HEIGHT * 3;
    uint8_t* frame_buffer = reinterpret_cast<uint8_t*>(malloc(frame_size));

    while (true)
    {
        bool draw = gg::tick(frame_buffer);
        if (draw)
        {
            QImage image(frame_buffer, 256, 224, QImage::Format_RGB888);
            QPixmap frame = QPixmap::fromImage(image);
            emit frameGenerated(frame);
        }
    }

    free(frame_buffer);
}
