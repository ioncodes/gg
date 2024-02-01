#include "mainwindow.h"
#include "ui_mainwindow.h"

#include "gg.hpp"
#include "emulatorthread.h"

MainWindow::MainWindow(QWidget *parent)
    : QMainWindow(parent)
    , ui(new Ui::MainWindow)
{
    ui->setupUi(this);

    gg::load();
    gg::init();

    EmulatorThread* thread = new EmulatorThread;
    connect(thread, SIGNAL(frameGenerated(QPixmap)), SLOT(onFrameGenerated(QPixmap)));
    thread->start();
}

MainWindow::~MainWindow()
{
    delete ui;
}

void MainWindow::onFrameGenerated(QPixmap frame)
{
    ui->lbl_frame->setPixmap(frame);
}
