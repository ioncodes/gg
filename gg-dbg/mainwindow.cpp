#include "mainwindow.h"
#include "./ui_mainwindow.h"

#include <Windows.h>

MainWindow::MainWindow(QWidget *parent)
    : QMainWindow(parent)
    , ui(new Ui::MainWindow)
{
    ui->setupUi(this);

    HMODULE handle = LoadLibraryA("core_ffi.dll");
    void* gg_init = reinterpret_cast<void*>(GetProcAddress(handle, "gg_init"));
    void* gg_tick = reinterpret_cast<void*>(GetProcAddress(handle, "gg_tick"));
    qDebug("gg_init = %p, gg_tick = %p", gg_init, gg_tick);

    reinterpret_cast<void (*)()>(gg_init)();
    uint8_t* ptr = (uint8_t*)malloc(256 * 224 * 3);
    bool draw = false;
    do {
        draw = reinterpret_cast<bool (*)(uint8_t*)>(gg_tick)(ptr);
        qDebug("ptr = %p", ptr);
    } while(draw == false);

    /*
    size_t rgba_frame_size = 256 * 224 * 3;
    for (int i = 0; i < rgba_frame_size; i += 4) {
        qDebug("%x,%x,%x,%x", ptr[i], ptr[i + 1], ptr[i + 2], ptr[i + 3]);
    }
    */

    QImage image(ptr, 256, 224, QImage::Format_RGB888);
    QPixmap frame = QPixmap::fromImage(image);
    ui->lbl_frame->setPixmap(frame);

    free(ptr);
}

MainWindow::~MainWindow()
{
    delete ui;
}
