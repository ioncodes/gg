#include "mainwindow.h"
#include "ui_mainwindow.h"

#include "gg.hpp"

MainWindow::MainWindow(QWidget *parent)
    : QMainWindow(parent)
    , ui(new Ui::MainWindow)
    , thread(new EmulatorThread)
{
    ui->setupUi(this);

    gg::load();
    log("gg_init => %p", gg::init);
    log("gg_tick => %p", gg::tick);

    gg::init();

    connect(thread, SIGNAL(frameGenerated(QPixmap)), SLOT(onFrameGenerated(QPixmap)));
    connect(thread, SIGNAL(registersFetched(Registers)), SLOT(onRegistersFetched(Registers)));
    connect(thread, SIGNAL(pause()), SLOT(onPause()));
    thread->start();
}

MainWindow::~MainWindow()
{
    delete thread;
    delete ui;
}

void MainWindow::onFrameGenerated(QPixmap frame)
{
    ui->lbl_frame->setPixmap(frame);
}

void MainWindow::onRegistersFetched(Registers registers)
{
    log("a: %x", registers.a);
}

void MainWindow::log(const char* fmt, ...)
{
    va_list args;
    va_start(args, fmt);

    QString msg = QString::asprintf(fmt, args);
    QStringList logs = ui->lbl_logs->text().split("\n");
    logs.push_back(msg);
    if (logs.size() > 50)
        do {
            logs.pop_front();
        } while (logs.size() > 50);
    ui->lbl_logs->setText(logs.join("\n"));

    va_end(args);
}

void MainWindow::on_btn_pause_clicked()
{
    emit pause();
}

